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
///
/// Architecture FFI :
/// - whisper_wrapper.c encapsule la config de whisper_full_params (struct trop large pour Rust)
/// - Metal détecté à la compilation (arm64 macOS = Metal disponible)
/// - CoreML nécessite Xcode complet + recompilation avec WHISPER_COREML=ON

use anyhow::{anyhow, Result};

/// Params de transcription Whisper optimisés pour le français (ADR-002)
#[derive(Debug, Clone)]
pub struct WhisperParams {
    /// Langue forcée — "fr" pour les performances FR optimales
    pub language: String,
    /// Traduction vers l'anglais (désactivé par défaut)
    pub translate: bool,
    /// Seuil de non-parole — évite les hallucinations
    pub no_speech_threshold: f32,
}

impl Default for WhisperParams {
    fn default() -> Self {
        Self {
            language: "fr".to_string(),
            translate: false,
            no_speech_threshold: 0.6, // ADR-002
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
    use std::os::raw::{c_char, c_float, c_int};

    extern "C" {
        // Cycle de vie du contexte
        pub fn whisper_init_from_file(path_model: *const c_char) -> *mut std::ffi::c_void;
        pub fn whisper_free(ctx: *mut std::ffi::c_void);

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

        // Wrapper C (whisper_wrapper.c) — encapsule la config whisper_full_params
        // language: code ISO (ex: "fr"), NULL = auto-détection
        pub fn whisper_run(
            ctx: *mut std::ffi::c_void,
            language: *const c_char,
            translate: bool,
            samples: *const c_float,
            n_samples: c_int,
        ) -> c_int;
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
            // Metal disponible sur Apple Silicon (compilé avec GGML_METAL=ON)
            // CoreML non disponible (nécessite Xcode complet + WHISPER_COREML=ON)
            log::info!(
                "whisper.cpp chargé — Metal: true (Apple Silicon), CoreML: false (CLT seulement)"
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
            use std::ffi::{CStr, CString};

            let language_c = CString::new(params.language.as_str())?;

            // Transcription via le wrapper C (gère whisper_full_params)
            let ret = unsafe {
                ffi::whisper_run(
                    self.ptr,
                    language_c.as_ptr(),
                    params.translate,
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

/// Metal disponible sur Apple Silicon (compilé avec GGML_METAL=ON par build-whisper.sh)
pub fn is_metal_available() -> bool {
    cfg!(all(target_os = "macos", target_arch = "aarch64", whisper_native))
}

/// CoreML nécessite Xcode complet + WHISPER_COREML=ON — non actif dans le build actuel
pub fn is_coreml_available() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_params_fr() {
        let p = WhisperParams::default();
        assert_eq!(p.language, "fr");
        assert!(!p.translate);
        assert_eq!(p.no_speech_threshold, 0.6);
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
    fn availability_checks() {
        // Stubs — en mode non-natif, tout est false
        #[cfg(not(whisper_native))]
        {
            assert!(!is_coreml_available());
            assert!(!is_metal_available());
        }
        // En mode natif sur Apple Silicon, Metal est disponible
        #[cfg(all(whisper_native, target_arch = "aarch64"))]
        {
            assert!(is_metal_available());
            assert!(!is_coreml_available()); // pas de Xcode complet
        }
    }
}
