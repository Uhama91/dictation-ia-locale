/// Gestionnaire de transcription STT — Whisper uniquement, optimisé FR
///
/// Note architecture : ce module utilise transcribe-rs/WhisperEngine comme base.
/// En Task 3-5, il sera migré vers whisper.cpp FFI direct (CoreML encoder + Metal decoder)
/// pour accéder à l'ANE et réduire la latence de ~450ms → ~200ms sur l'encodeur.
use crate::audio_toolkit::{apply_custom_words, filter_transcription_output};
use crate::managers::model::ModelManager;
use crate::settings::{get_settings, ModelUnloadTimeout};
use anyhow::Result;
use log::{debug, error, info, warn};
use serde::Serialize;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, SystemTime};
use tauri::{AppHandle, Emitter};
use transcribe_rs::{
    engines::whisper::{WhisperEngine, WhisperInferenceParams},
    TranscriptionEngine,
};

#[derive(Clone, Debug, Serialize)]
pub struct ModelStateEvent {
    pub event_type: String,
    pub model_id: Option<String>,
    pub model_name: Option<String>,
    pub error: Option<String>,
}

/// Résultat de transcription enrichi avec métadonnées pour le pipeline hybride
#[derive(Clone, Debug)]
pub struct TranscriptionOutput {
    pub text: String,
    /// Score de confiance [0.0, 1.0] — utilisé par pipeline/orchestrator.rs
    /// pour router vers règles seules ou LLM conditionnel
    pub confidence: f32,
    pub duration_ms: u64,
}

enum LoadedEngine {
    /// transcribe-rs WhisperEngine (fallback, toujours disponible)
    Whisper(WhisperEngine),
    /// whisper.cpp FFI natif (activé quand whisper_native cfg = true)
    /// Offre CoreML encoder (ANE) + Metal decoder — latence ~2-3x inférieure
    #[cfg(whisper_native)]
    WhisperFfi(crate::whisper_ffi::WhisperContext),
}

#[derive(Clone)]
pub struct TranscriptionManager {
    engine: Arc<Mutex<Option<LoadedEngine>>>,
    model_manager: Arc<ModelManager>,
    app_handle: AppHandle,
    current_model_id: Arc<Mutex<Option<String>>>,
    last_activity: Arc<AtomicU64>,
    shutdown_signal: Arc<AtomicBool>,
    watcher_handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
    is_loading: Arc<Mutex<bool>>,
    loading_condvar: Arc<Condvar>,
}

impl TranscriptionManager {
    pub fn new(app_handle: &AppHandle, model_manager: Arc<ModelManager>) -> Result<Self> {
        let manager = Self {
            engine: Arc::new(Mutex::new(None)),
            model_manager,
            app_handle: app_handle.clone(),
            current_model_id: Arc::new(Mutex::new(None)),
            last_activity: Arc::new(AtomicU64::new(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            )),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            watcher_handle: Arc::new(Mutex::new(None)),
            is_loading: Arc::new(Mutex::new(false)),
            loading_condvar: Arc::new(Condvar::new()),
        };

        // Démarrer le watcher de déchargement par inactivité
        {
            let app_handle_cloned = app_handle.clone();
            let manager_cloned = manager.clone();
            let shutdown_signal = manager.shutdown_signal.clone();
            let handle = thread::spawn(move || {
                while !shutdown_signal.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_secs(10));

                    if shutdown_signal.load(Ordering::Relaxed) {
                        break;
                    }

                    let settings = get_settings(&app_handle_cloned);
                    let timeout_seconds = settings.model_unload_timeout.to_seconds();

                    if let Some(limit_seconds) = timeout_seconds {
                        if settings.model_unload_timeout == ModelUnloadTimeout::Immediately {
                            continue;
                        }

                        let last = manager_cloned.last_activity.load(Ordering::Relaxed);
                        let now_ms = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64;

                        if now_ms.saturating_sub(last) > limit_seconds * 1000 {
                            if manager_cloned.is_model_loaded() {
                                if let Ok(()) = manager_cloned.unload_model() {
                                    let _ = app_handle_cloned.emit(
                                        "model-state-changed",
                                        ModelStateEvent {
                                            event_type: "unloaded".to_string(),
                                            model_id: None,
                                            model_name: None,
                                            error: None,
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
                debug!("Idle watcher thread shutting down gracefully");
            });
            *manager.watcher_handle.lock().unwrap() = Some(handle);
        }

        Ok(manager)
    }

    fn lock_engine(&self) -> MutexGuard<'_, Option<LoadedEngine>> {
        self.engine.lock().unwrap_or_else(|poisoned| {
            warn!("Engine mutex was poisoned by a previous panic, recovering");
            poisoned.into_inner()
        })
    }

    pub fn is_model_loaded(&self) -> bool {
        self.lock_engine().is_some()
    }

    pub fn unload_model(&self) -> Result<()> {
        {
            let mut engine = self.lock_engine();
            if let Some(ref mut loaded_engine) = *engine {
                match loaded_engine {
                    LoadedEngine::Whisper(ref mut e) => e.unload_model(),
                    // WhisperFfi se décharge via Drop automatiquement
                    #[cfg(whisper_native)]
                    LoadedEngine::WhisperFfi(_) => {}
                }
            }
            *engine = None;
        }
        {
            let mut current_model = self.current_model_id.lock().unwrap();
            *current_model = None;
        }

        let _ = self.app_handle.emit(
            "model-state-changed",
            ModelStateEvent {
                event_type: "unloaded".to_string(),
                model_id: None,
                model_name: None,
                error: None,
            },
        );
        Ok(())
    }

    pub fn maybe_unload_immediately(&self, context: &str) {
        let settings = get_settings(&self.app_handle);
        if settings.model_unload_timeout == ModelUnloadTimeout::Immediately
            && self.is_model_loaded()
        {
            info!("Immediately unloading model after {}", context);
            if let Err(e) = self.unload_model() {
                warn!("Failed to immediately unload model: {}", e);
            }
        }
    }

    pub fn load_model(&self, model_id: &str) -> Result<()> {
        let load_start = std::time::Instant::now();
        debug!("Loading STT model: {}", model_id);

        let _ = self.app_handle.emit(
            "model-state-changed",
            ModelStateEvent {
                event_type: "loading_started".to_string(),
                model_id: Some(model_id.to_string()),
                model_name: None,
                error: None,
            },
        );

        let model_info = self
            .model_manager
            .get_model_info(model_id)
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_id))?;

        if !model_info.is_downloaded {
            let error_msg = "Model not downloaded".to_string();
            let _ = self.app_handle.emit(
                "model-state-changed",
                ModelStateEvent {
                    event_type: "loading_failed".to_string(),
                    model_id: Some(model_id.to_string()),
                    model_name: Some(model_info.name.clone()),
                    error: Some(error_msg.clone()),
                },
            );
            return Err(anyhow::anyhow!(error_msg));
        }

        let model_path = self.model_manager.get_model_path(model_id)?;

        // Essayer whisper_ffi natif (CoreML + Metal) si disponible
        #[cfg(whisper_native)]
        {
            match crate::whisper_ffi::WhisperContext::load(&model_path) {
                Ok(ctx) => {
                    info!(
                        "whisper.cpp natif chargé — CoreML: {}, Metal: {}",
                        crate::whisper_ffi::is_coreml_available(),
                        crate::whisper_ffi::is_metal_available()
                    );
                    let mut engine_guard = self.lock_engine();
                    *engine_guard = Some(LoadedEngine::WhisperFfi(ctx));
                    let mut current_model = self.current_model_id.lock().unwrap();
                    *current_model = Some(model_id.to_string());
                    drop(current_model);
                    drop(engine_guard);
                    let _ = self.app_handle.emit(
                        "model-state-changed",
                        ModelStateEvent {
                            event_type: "loading_completed".to_string(),
                            model_id: Some(model_id.to_string()),
                            model_name: Some(model_info.name.clone()),
                            error: None,
                        },
                    );
                    debug!(
                        "STT model loaded via whisper_ffi in {}ms: {}",
                        load_start.elapsed().as_millis(),
                        model_id
                    );
                    return Ok(());
                }
                Err(e) => {
                    warn!("whisper_ffi::load() échoué, fallback transcribe-rs: {}", e);
                }
            }
        }

        // Fallback : transcribe-rs WhisperEngine
        let mut engine = WhisperEngine::new();
        engine.load_model(&model_path).map_err(|e| {
            let error_msg = format!("Failed to load whisper model {}: {}", model_id, e);
            let _ = self.app_handle.emit(
                "model-state-changed",
                ModelStateEvent {
                    event_type: "loading_failed".to_string(),
                    model_id: Some(model_id.to_string()),
                    model_name: Some(model_info.name.clone()),
                    error: Some(error_msg.clone()),
                },
            );
            anyhow::anyhow!(error_msg)
        })?;

        {
            let mut engine_guard = self.lock_engine();
            *engine_guard = Some(LoadedEngine::Whisper(engine));
        }
        {
            let mut current_model = self.current_model_id.lock().unwrap();
            *current_model = Some(model_id.to_string());
        }

        let _ = self.app_handle.emit(
            "model-state-changed",
            ModelStateEvent {
                event_type: "loading_completed".to_string(),
                model_id: Some(model_id.to_string()),
                model_name: Some(model_info.name.clone()),
                error: None,
            },
        );

        debug!(
            "STT model loaded in {}ms: {}",
            load_start.elapsed().as_millis(),
            model_id
        );
        Ok(())
    }

    pub fn initiate_model_load(&self) {
        let mut is_loading = self.is_loading.lock().unwrap();
        if *is_loading || self.is_model_loaded() {
            return;
        }

        *is_loading = true;
        let self_clone = self.clone();
        thread::spawn(move || {
            let settings = get_settings(&self_clone.app_handle);
            if let Err(e) = self_clone.load_model(&settings.selected_model) {
                error!("Failed to load model: {}", e);
            }
            let mut is_loading = self_clone.is_loading.lock().unwrap();
            *is_loading = false;
            self_clone.loading_condvar.notify_all();
        });
    }

    pub fn get_current_model(&self) -> Option<String> {
        self.current_model_id.lock().unwrap().clone()
    }

    /// Transcrit l'audio et retourne le texte + métadonnées pour le pipeline hybride
    ///
    /// Params FR optimisés (ADR-002) :
    /// - language: "fr" (forcé — pas d'auto-detect pour réduire la latence)
    /// - beam_size: 1, temperature: 0.0 → greedy decoding, plus rapide
    ///
    /// TODO Task 3-5 : remplacer WhisperEngine par whisper_ffi::WhisperContext
    /// pour accéder au CoreML encoder (ANE 3x) + Metal decoder (3-4x)
    pub fn transcribe(&self, audio: Vec<f32>) -> Result<TranscriptionOutput> {
        self.last_activity.store(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            Ordering::Relaxed,
        );

        let st = std::time::Instant::now();

        if audio.is_empty() {
            self.maybe_unload_immediately("empty audio");
            return Ok(TranscriptionOutput {
                text: String::new(),
                confidence: 1.0,
                duration_ms: 0,
            });
        }

        {
            let mut is_loading = self.is_loading.lock().unwrap();
            while *is_loading {
                is_loading = self.loading_condvar.wait(is_loading).unwrap();
            }

            let engine_guard = self.lock_engine();
            if engine_guard.is_none() {
                return Err(anyhow::anyhow!("Model is not loaded for transcription."));
            }
        }

        let settings = get_settings(&self.app_handle);

        let result = {
            let mut engine_guard = self.lock_engine();
            let mut engine = match engine_guard.take() {
                Some(e) => e,
                None => {
                    return Err(anyhow::anyhow!("Model not loaded. Check settings."));
                }
            };
            drop(engine_guard);

            // Retourne (text, no_speech_prob_optionnel)
            // - WhisperFfi : no_speech_prob réel depuis whisper.cpp
            // - Whisper    : None (heuristique utilisée plus bas)
            let transcribe_result: std::thread::Result<Result<(String, Option<f32>)>> =
                catch_unwind(AssertUnwindSafe(|| {
                    match &mut engine {
                        #[cfg(whisper_native)]
                        LoadedEngine::WhisperFfi(ctx) => {
                            // Chemin natif : whisper.cpp CoreML encoder (ANE) + Metal decoder
                            let params = crate::whisper_ffi::WhisperParams {
                                language: if settings.selected_language == "auto" {
                                    "fr".to_string()
                                } else {
                                    settings.selected_language.clone()
                                },
                                translate: settings.translate_to_english,
                                ..Default::default()
                            };
                            ctx.transcribe(&audio, &params)
                                .map(|r| (r.text, Some(r.no_speech_prob)))
                                .map_err(|e| anyhow::anyhow!("whisper_ffi failed: {}", e))
                        }
                        LoadedEngine::Whisper(whisper_engine) => {
                            // Fallback : transcribe-rs (ADR-002 greedy FR)
                            let params = WhisperInferenceParams {
                                language: if settings.selected_language == "auto" {
                                    Some("fr".to_string())
                                } else {
                                    Some(settings.selected_language.clone())
                                },
                                translate: settings.translate_to_english,
                                ..Default::default()
                            };
                            whisper_engine
                                .transcribe_samples(audio, Some(params))
                                .map(|o| (o.text, None::<f32>))
                                .map_err(|e| anyhow::anyhow!("Whisper transcription failed: {}", e))
                        }
                    }
                }));

            match transcribe_result {
                Ok(inner_result) => {
                    let mut engine_guard = self.lock_engine();
                    *engine_guard = Some(engine);
                    inner_result?
                }
                Err(panic_payload) => {
                    let panic_msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".to_string()
                    };
                    error!("Transcription engine panicked: {}. Model unloaded.", panic_msg);
                    {
                        let mut current_model = self
                            .current_model_id
                            .lock()
                            .unwrap_or_else(|e| e.into_inner());
                        *current_model = None;
                    }
                    let _ = self.app_handle.emit(
                        "model-state-changed",
                        ModelStateEvent {
                            event_type: "unloaded".to_string(),
                            model_id: None,
                            model_name: None,
                            error: Some(format!("Engine panicked: {}", panic_msg)),
                        },
                    );
                    return Err(anyhow::anyhow!(
                        "Transcription engine panicked: {}. Model reloaded on next attempt.",
                        panic_msg
                    ));
                }
            }
        };

        // result = (text: String, no_speech_prob: Option<f32>)
        let (raw_text, no_speech_prob_opt) = result;

        // Correction des mots personnalisés
        let corrected = if !settings.custom_words.is_empty() {
            apply_custom_words(
                &raw_text,
                &settings.custom_words,
                settings.word_correction_threshold,
            )
        } else {
            raw_text
        };

        // Filtrage filler words et hallucinations
        let filtered = filter_transcription_output(&corrected);

        let duration_ms = st.elapsed().as_millis() as u64;
        info!("Transcription FR terminée en {}ms", duration_ms);

        if filtered.is_empty() {
            info!("Transcription result is empty");
        }

        self.maybe_unload_immediately("transcription");

        // Calcul du score de confiance :
        // - whisper_ffi : 1.0 - no_speech_prob (score réel depuis whisper.cpp)
        // - transcribe-rs fallback : heuristique longueur (jusqu'à Task 3-4)
        let confidence = compute_confidence(&filtered, no_speech_prob_opt);

        Ok(TranscriptionOutput {
            text: filtered,
            confidence,
            duration_ms,
        })
    }

    /// API compatible avec l'existant — retourne seulement le texte
    pub fn transcribe_text(&self, audio: Vec<f32>) -> Result<String> {
        self.transcribe(audio).map(|o| o.text)
    }
}

impl Drop for TranscriptionManager {
    fn drop(&mut self) {
        self.shutdown_signal.store(true, Ordering::Relaxed);
        if let Some(handle) = self.watcher_handle.lock().unwrap().take() {
            if let Err(e) = handle.join() {
                warn!("Failed to join idle watcher thread: {:?}", e);
            }
        }
    }
}

/// Calcule le score de confiance de transcription (extrait pour testabilité).
///
/// - whisper_ffi (chemin natif) : `1.0 - no_speech_prob` depuis whisper.cpp
/// - transcribe-rs fallback : heuristique word_count
///   - texte vide → 0.0
///   - <= 30 mots → 0.90 (phrases courtes → confiance élevée, fast-path rules)
///   - > 30 mots → 0.75 (phrases longues → LLM conditionnel)
pub fn compute_confidence(text: &str, no_speech_prob: Option<f32>) -> f32 {
    if text.is_empty() {
        0.0
    } else if let Some(nsp) = no_speech_prob {
        (1.0 - nsp).clamp(0.0, 1.0)
    } else if text.split_whitespace().count() <= 30 {
        0.90
    } else {
        0.75
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_empty_text_is_zero() {
        assert_eq!(compute_confidence("", None), 0.0);
        assert_eq!(compute_confidence("", Some(0.1)), 0.0);
    }

    #[test]
    fn confidence_from_no_speech_prob_real_score() {
        // Chemin whisper_ffi : confiance = 1.0 - no_speech_prob
        let c = compute_confidence("bonjour le monde", Some(0.1));
        assert!((c - 0.9).abs() < 1e-6, "expected ~0.9, got {c}");

        let c_high = compute_confidence("test", Some(0.05));
        assert!((c_high - 0.95).abs() < 1e-6);

        // Valeur extrême clampée à 1.0
        let c_max = compute_confidence("test", Some(0.0));
        assert_eq!(c_max, 1.0);
    }

    #[test]
    fn confidence_heuristic_short_phrase() {
        // Fallback transcribe-rs : <= 30 mots → 0.90 (fast-path rules)
        let text = "bonjour je voudrais commander une pizza";
        assert_eq!(compute_confidence(text, None), 0.90);
    }

    #[test]
    fn confidence_heuristic_long_phrase() {
        // > 30 mots → 0.75 (LLM conditionnel)
        let text = "un deux trois quatre cinq six sept huit neuf dix onze douze treize quatorze \
                    quinze seize dix-sept dix-huit dix-neuf vingt vingt-et-un vingt-deux \
                    vingt-trois vingt-quatre vingt-cinq vingt-six vingt-sept vingt-huit \
                    vingt-neuf trente trente-et-un";
        let word_count = text.split_whitespace().count();
        assert!(word_count > 30, "phrase de test trop courte: {word_count} mots");
        assert_eq!(compute_confidence(text, None), 0.75);
    }

    #[test]
    fn confidence_heuristic_boundary_30_words() {
        // Exactement 30 mots → 0.90 (inclus dans fast-path)
        let text: String = (0..30).map(|_| "mot").collect::<Vec<_>>().join(" ");
        assert_eq!(text.split_whitespace().count(), 30);
        assert_eq!(compute_confidence(&text, None), 0.90);
    }
}
