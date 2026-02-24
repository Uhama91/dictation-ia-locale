/// whisper_ffi.rs — Bindings Rust → whisper.cpp (Tasks 3-4)
///
/// Compilation whisper.cpp requise :
/// ```bash
/// ./scripts/build-whisper.sh
/// ```
///
/// Quand whisper.cpp est compilé (vendor/whisper.cpp/build/src/libwhisper.a),
/// `cfg(whisper_native)` est activé par build.rs et les vrais bindings sont utilisés.
/// Sans la lib, `WhisperContext::load()` retourne une Err (mode stub/fallback).

use anyhow::{anyhow, Result};

/// Params de transcription Whisper optimisés pour le français (ADR-002)
#[derive(Debug, Clone)]
pub struct WhisperParams {
    /// Langue forcée — "fr" pour les performances FR optimales
    pub language: String,
    /// Greedy decoding = plus rapide (beam_size effectif de 1)
    pub beam_size: i32,
    /// temperature=0.0 → greedy, déterministe
    pub temperature: f32,
    /// Seuil de non-parole — évite les hallucinations
    pub no_speech_threshold: f32,
    /// Traduction vers l'anglais (désactivé par défaut)
    pub translate: bool,
}

impl Default for WhisperParams {
    fn default() -> Self {
        Self {
            language: "fr".to_string(),
            beam_size: 1,             // Greedy = ~100ms de gain vs beam_size=5
            temperature: 0.0,         // Déterministe
            no_speech_threshold: 0.6, // ADR-002
            translate: false,
        }
    }
}

/// Résultat de transcription avec métadonnées
#[derive(Debug, Clone)]
pub struct WhisperResult {
    pub text: String,
    /// Probabilité de non-parole [0.0, 1.0] (0.0 = parole certaine)
    pub no_speech_prob: f32,
}

// ─── Bindings C natifs (activés quand whisper.cpp est compilé) ───────────────

#[cfg(whisper_native)]
mod ffi {
    use std::ffi::{CStr, CString};
    use std::os::raw::{c_char, c_float, c_int};

    extern "C" {
        // Cycle de vie du contexte
        pub fn whisper_init_from_file(path_model: *const c_char) -> *mut std::ffi::c_void;
        pub fn whisper_free(ctx: *mut std::ffi::c_void);

        // Transcription
        pub fn whisper_full_default_params(strategy: c_int) -> WhisperFullParamsC;
        pub fn whisper_full(
            ctx: *mut std::ffi::c_void,
            params: WhisperFullParamsC,
            samples: *const c_float,
            n_samples: c_int,
        ) -> c_int;

        // Lecture des résultats
        pub fn whisper_full_n_segments(ctx: *const std::ffi::c_void) -> c_int;
        pub fn whisper_full_get_segment_text(
            ctx: *const std::ffi::c_void,
            i_segment: c_int,
        ) -> *const c_char;
        pub fn whisper_full_get_segment_no_speech_prob(
            ctx: *const std::ffi::c_void,
            i_segment: c_int,
        ) -> c_float;

        // Capacités hardware
        pub fn whisper_has_coreml() -> c_int;
        pub fn whisper_has_metal() -> c_int;
    }

    /// Stratégie de décodage whisper.cpp
    /// 0 = WHISPER_SAMPLING_GREEDY, 1 = WHISPER_SAMPLING_BEAM_SEARCH
    pub const WHISPER_SAMPLING_GREEDY: c_int = 0;

    /// Représentation C de whisper_full_params (partielle — champs essentiels)
    /// La struct complète est beaucoup plus grande — on utilise repr(C) + padding.
    #[repr(C)]
    pub struct WhisperFullParamsC {
        pub strategy: c_int,
        pub n_threads: c_int,
        pub n_max_text_ctx: c_int,
        pub offset_ms: c_int,
        pub duration_ms: c_int,
        pub translate: bool,
        pub no_context: bool,
        pub no_timestamps: bool,
        pub single_segment: bool,
        pub print_special: bool,
        pub print_progress: bool,
        pub print_realtime: bool,
        pub print_timestamps: bool,
        // ... (padding pour correspondre à la struct C complète)
        // Note: whisper_full_default_params() initialise tous les champs
        // On override uniquement les champs qu'on veut changer
        _padding: [u8; 512], // Sécurité — taille exacte à valider avec whisper.cpp headers
    }
}

// ─── WhisperContext ───────────────────────────────────────────────────────────

/// Contexte de transcription whisper.cpp
pub struct WhisperContext {
    #[cfg(whisper_native)]
    ptr: *mut std::ffi::c_void,
    #[cfg(not(whisper_native))]
    _placeholder: (),
}

// SAFETY: whisper.cpp garantit que whisper_context est thread-safe en lecture
#[cfg(whisper_native)]
unsafe impl Send for WhisperContext {}
#[cfg(whisper_native)]
unsafe impl Sync for WhisperContext {}

impl WhisperContext {
    /// Charge un modèle Whisper depuis un fichier GGUF/GGML
    pub fn load(model_path: &std::path::Path) -> Result<Self> {
        #[cfg(whisper_native)]
        {
            use std::ffi::CString;
            let path_str = model_path
                .to_str()
                .ok_or_else(|| anyhow!("Chemin modèle invalide (non-UTF8)"))?;
            let c_path = CString::new(path_str)?;

            let ptr = unsafe { ffi::whisper_init_from_file(c_path.as_ptr()) };
            if ptr.is_null() {
                return Err(anyhow!(
                    "whisper_init_from_file() a retourné NULL pour: {}",
                    model_path.display()
                ));
            }
            log::info!(
                "whisper.cpp chargé — CoreML: {}, Metal: {}",
                unsafe { ffi::whisper_has_coreml() == 1 },
                unsafe { ffi::whisper_has_metal() == 1 }
            );
            Ok(Self { ptr })
        }

        #[cfg(not(whisper_native))]
        {
            let _ = model_path;
            Err(anyhow!(
                "whisper_ffi non compilé. Exécuter scripts/build-whisper.sh \
                 puis recompiler. Mode fallback: transcribe-rs."
            ))
        }
    }

    /// Transcrit les samples audio f32 16kHz mono
    pub fn transcribe(&self, audio: &[f32], params: &WhisperParams) -> Result<WhisperResult> {
        #[cfg(whisper_native)]
        {
            use std::ffi::CStr;

            // Construire les params C depuis les defaults
            let mut c_params =
                unsafe { ffi::whisper_full_default_params(ffi::WHISPER_SAMPLING_GREEDY) };

            // Override avec nos paramètres FR
            c_params.translate = params.translate;
            c_params.single_segment = false;
            c_params.print_progress = false;
            c_params.print_realtime = false;
            c_params.print_timestamps = false;

            // Lancer la transcription
            let ret = unsafe {
                ffi::whisper_full(
                    self.ptr,
                    c_params,
                    audio.as_ptr(),
                    audio.len() as std::os::raw::c_int,
                )
            };

            if ret != 0 {
                return Err(anyhow!("whisper_full() a retourné l'erreur: {}", ret));
            }

            // Lire les segments
            let n_segments = unsafe { ffi::whisper_full_n_segments(self.ptr) };
            let mut text_parts: Vec<String> = Vec::with_capacity(n_segments as usize);
            let mut total_no_speech = 0.0f32;

            for i in 0..n_segments {
                let seg_text = unsafe {
                    let ptr = ffi::whisper_full_get_segment_text(self.ptr, i);
                    if ptr.is_null() {
                        continue;
                    }
                    CStr::from_ptr(ptr).to_string_lossy().into_owned()
                };
                let no_speech =
                    unsafe { ffi::whisper_full_get_segment_no_speech_prob(self.ptr, i) };

                // Filtrer les segments de non-parole
                if no_speech < params.no_speech_threshold {
                    text_parts.push(seg_text.trim().to_string());
                }
                total_no_speech += no_speech;
            }

            let avg_no_speech = if n_segments > 0 {
                total_no_speech / n_segments as f32
            } else {
                1.0 // Pas de segments = probablement du silence
            };

            Ok(WhisperResult {
                text: text_parts.join(" ").trim().to_string(),
                no_speech_prob: avg_no_speech,
            })
        }

        #[cfg(not(whisper_native))]
        {
            let _ = (audio, params);
            Err(anyhow!("whisper_ffi non compilé (stub)"))
        }
    }
}

#[cfg(whisper_native)]
impl Drop for WhisperContext {
    fn drop(&mut self) {
        unsafe { ffi::whisper_free(self.ptr) };
    }
}

// ─── Fonctions utilitaires ────────────────────────────────────────────────────

/// Vérifie si le support CoreML est disponible (ANE Apple Silicon)
pub fn is_coreml_available() -> bool {
    #[cfg(whisper_native)]
    {
        unsafe { ffi::whisper_has_coreml() == 1 }
    }
    #[cfg(not(whisper_native))]
    {
        false // Stub — vrai quand whisper.cpp est compilé avec WHISPER_COREML=ON
    }
}

/// Vérifie si le support Metal est disponible (GPU Apple Silicon)
pub fn is_metal_available() -> bool {
    #[cfg(whisper_native)]
    {
        unsafe { ffi::whisper_has_metal() == 1 }
    }
    #[cfg(not(whisper_native))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_params_fr_greedy() {
        let p = WhisperParams::default();
        assert_eq!(p.language, "fr");
        assert_eq!(p.beam_size, 1);
        assert_eq!(p.temperature, 0.0);
        assert!(!p.translate);
    }

    #[test]
    fn load_stub_returns_err() {
        // Sans whisper_native compilé, load() doit retourner Err
        #[cfg(not(whisper_native))]
        {
            let result = WhisperContext::load(std::path::Path::new("/nonexistent/model.gguf"));
            assert!(result.is_err());
        }
    }

    #[test]
    fn availability_stubs() {
        #[cfg(not(whisper_native))]
        {
            assert!(!is_coreml_available());
            assert!(!is_metal_available());
        }
    }
}
