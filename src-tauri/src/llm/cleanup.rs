/// LLM de nettoyage FR — Qwen2.5-0.5B Q4 via llama.cpp
///
/// Task 10 — implémentation complète à venir
///
/// Architecture prévue :
/// - Modèle : Qwen2.5-0.5B-Instruct Q4_K_M (~300 Mo RAM)
/// - Backend : llama-cpp-2 crate (bindings Rust pour llama.cpp)
/// - Params : n_predict=128, temperature=0.0, top_k=1, repeat_penalty=1.0
/// - Chargement à la demande + déchargement après 5min d'inactivité
///
/// Pour l'instant ce module est un stub — le pipeline utilise les règles seules.

use crate::pipeline::modes::WriteMode;
use anyhow::Result;

/// État du modèle LLM
#[derive(Debug, Clone, PartialEq)]
pub enum LlmState {
    /// Modèle non chargé (état initial ou après déchargement)
    Unloaded,
    /// Modèle en cours de chargement
    Loading,
    /// Modèle prêt à l'inférence
    Ready,
    /// Erreur de chargement
    Error(String),
}

/// Nettoie le texte transcrit avec le LLM Qwen2.5-0.5B Q4.
///
/// # Arguments
/// * `text` - Texte brut post-règles à nettoyer
/// * `mode` - Mode d'écriture (Chat/Pro/Code) — détermine le prompt système
///
/// # Returns
/// Texte nettoyé, ou erreur si le modèle n'est pas disponible
///
/// TODO Task 10 : Implémenter avec llama-cpp-2 :
/// ```rust
/// // Exemple d'implémentation prévue :
/// // let model = LlamaModel::load_from_file(&model_path, LlamaParams::default())?;
/// // let ctx = LlamaContext::new(&model, LlamaContextParams::default())?;
/// // Construire le prompt avec mode.system_prompt() + text
/// // Inférer avec n_predict=128, temperature=0.0, top_k=1
/// ```
pub fn run(text: &str, mode: WriteMode) -> Result<String> {
    // STUB — le LLM n'est pas encore implémenté
    // Retourne le texte tel quel (les règles ont déjà été appliquées)
    log::debug!(
        "LLM cleanup stub called (mode: {}, {} words) — returning rules-only result",
        mode,
        text.split_whitespace().count()
    );

    // TODO : remplacer par l'inférence llama.cpp réelle
    Err(anyhow::anyhow!(
        "LLM not yet implemented (Task 10). Pipeline fallback to rules-only."
    ))
}

/// Vérifie si le modèle LLM est disponible sur le système.
/// Cherche le fichier GGUF dans les emplacements standard.
///
/// TODO Task 10 : Implémenter la vérification du chemin modèle
pub fn is_model_available() -> bool {
    // TODO : vérifier dans app_data_dir/models/qwen2.5-0.5b-q4_k_m.gguf
    false
}
