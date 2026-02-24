/// whisper_ffi.rs — Bindings Rust → whisper.cpp (Task 4)
///
/// Ce module sera le remplacement de transcribe-rs pour accéder directement
/// aux optimisations CoreML encoder (ANE) + Metal decoder de whisper.cpp.
///
/// Pourquoi remplacer transcribe-rs (ADR-002) :
/// - transcribe-rs ne permet pas d'accéder aux optimisations WHISPER_COREML=1
/// - La compilation avec COREML+METAL est nécessaire pour ANE (3x encoder)
/// - Accès direct à l'API C de whisper.cpp = plus de contrôle sur les params
///
/// Compilation whisper.cpp requise (Task 3) :
/// ```bash
/// cmake -B build -DWHISPER_COREML=1 -DWHISPER_METAL=1
/// cmake --build build -j $(nproc)
/// ```
///
/// Les modèles CoreML doivent être générés avec :
/// ```bash
/// python3 models/generate_coreml_model.py large-v3-turbo
/// ```
///
/// TODO Task 3 : Intégrer whisper.cpp comme dépendance native dans build.rs
/// TODO Task 4 : Implémenter les bindings unsafe C → Rust

use anyhow::Result;

/// Params de transcription Whisper optimisés FR (ADR-002)
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
            beam_size: 1,          // Greedy = ~100ms de gain vs beam_size=5
            temperature: 0.0,      // Déterministe
            no_speech_threshold: 0.6,
            translate: false,
        }
    }
}

/// Résultat de transcription avec métadonnées
#[derive(Debug, Clone)]
pub struct WhisperResult {
    pub text: String,
    /// Probabilité de non-parole [0.0, 1.0]
    pub no_speech_prob: f32,
}

/// Context de transcription whisper.cpp
///
/// TODO Task 4 : Wrapper around whisper_context de whisper.cpp
/// ```c
/// struct whisper_context * whisper_init_from_file(const char * path_model);
/// int whisper_full(struct whisper_context * ctx, struct whisper_full_params params,
///                  const float * samples, int n_samples);
/// const char * whisper_full_get_segment_text(struct whisper_context * ctx, int i_segment);
/// void whisper_free(struct whisper_context * ctx);
/// ```
pub struct WhisperContext {
    // TODO Task 4 : ptr: *mut c_void (whisper_context*)
    _placeholder: (),
}

impl WhisperContext {
    /// Charge un modèle Whisper depuis un fichier GGUF/GGML
    ///
    /// TODO Task 4 : Appeler whisper_init_from_file() via FFI
    pub fn load(_model_path: &std::path::Path) -> Result<Self> {
        Err(anyhow::anyhow!(
            "whisper_ffi non encore implémenté (Task 3-4). \
             Utiliser transcribe-rs WhisperEngine en attendant."
        ))
    }

    /// Transcrit les samples audio f32 16kHz
    ///
    /// TODO Task 4 : Appeler whisper_full() + extraire les segments
    pub fn transcribe(&self, _audio: &[f32], _params: &WhisperParams) -> Result<WhisperResult> {
        Err(anyhow::anyhow!("whisper_ffi non encore implémenté (Task 4)"))
    }
}

/// Vérifie si le support CoreML est disponible (ANE)
///
/// TODO Task 4 : Appeler whisper_has_coreml() via FFI
pub fn is_coreml_available() -> bool {
    // Sur Apple Silicon, CoreML/ANE est toujours disponible si compilé avec WHISPER_COREML=1
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        // TODO : vérifier la compilation whisper.cpp avec COREML
        false // Stub — true quand Task 3-4 sera implémenté
    }
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    {
        false
    }
}
