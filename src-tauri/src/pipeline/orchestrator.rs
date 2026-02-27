/// Orchestrateur du pipeline hybride STT → Règles → [LLM conditionnel]
///
/// Task 12 : Routing conditionnel basé sur le score de confiance Whisper
///
/// Seuils de routing (configurables) :
/// - confidence >= 0.82 ET words <= 30 ET mode != Pro → règles seules (< 5ms)
/// - Sinon → règles + LLM Qwen2.5-0.5B Q4 (~200-300ms)
///
/// Résultat attendu : 60-65% des cas sans LLM

use crate::pipeline::modes::WriteMode;
use crate::pipeline::rules;

/// Seuil de confiance au-dessus duquel on évite le LLM (mode Chat/Code)
const CONFIDENCE_THRESHOLD: f32 = 0.82;
/// Nombre max de mots pour le fast-path sans LLM
const MAX_WORDS_FAST_PATH: usize = 30;

/// Résultat du pipeline post-traitement
#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub text: String,
    /// true = traité par règles seules, false = LLM utilisé
    pub rules_only: bool,
    /// true = le LLM était requis mais a échoué (fallback sur règles)
    pub llm_fallback: bool,
    pub duration_ms: u64,
}

/// Décision de routing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoutingDecision {
    RulesOnly,
    RulesAndLlm,
}

/// Détermine si on peut éviter le LLM
pub fn route(confidence: f32, word_count: usize, mode: WriteMode) -> RoutingDecision {
    // Le mode Pro reformule toujours → LLM obligatoire
    if mode.always_use_llm() {
        return RoutingDecision::RulesAndLlm;
    }

    // Fast path : confiance élevée + texte court
    if confidence >= CONFIDENCE_THRESHOLD && word_count <= MAX_WORDS_FAST_PATH {
        RoutingDecision::RulesOnly
    } else {
        RoutingDecision::RulesAndLlm
    }
}

/// Exécute le pipeline de post-traitement.
///
/// `llm_cleanup_fn` est un callback optionnel vers le LLM (None = LLM non encore disponible).
/// Quand None, on applique seulement les règles quel que soit le routing.
///
/// TODO Task 10 : brancher `crate::llm::cleanup::run` ici
pub fn process(
    raw_text: &str,
    confidence: f32,
    mode: WriteMode,
    llm_cleanup_fn: Option<&dyn Fn(&str, WriteMode) -> anyhow::Result<String>>,
) -> PipelineResult {
    let start = std::time::Instant::now();

    // Étape 1 : règles locales (toujours)
    let rules_result = rules::apply(raw_text);

    let word_count = rules_result.split_whitespace().count();
    let decision = route(confidence, word_count, mode);

    log::info!(
        "[Routing] confiance={:.2} mots={} mode={:?} → {}",
        confidence, word_count, mode,
        if decision == RoutingDecision::RulesOnly { "fast-path (règles)" } else { "LLM" }
    );

    // Étape 2 : LLM conditionnel
    let (final_text, rules_only, llm_fallback) = match (decision, llm_cleanup_fn) {
        (RoutingDecision::RulesAndLlm, Some(cleanup_fn)) => {
            match cleanup_fn(&rules_result, mode) {
                Ok(llm_result) => (llm_result, false, false),
                Err(e) => {
                    log::warn!("LLM cleanup failed, falling back to rules: {}", e);
                    (rules_result, true, true)
                }
            }
        }
        (RoutingDecision::RulesAndLlm, None) => {
            // LLM requis mais pas disponible
            (rules_result, true, true)
        }
        // Fast-path : règles seules (pas de fallback)
        _ => (rules_result, true, false),
    };

    PipelineResult {
        text: final_text,
        rules_only,
        llm_fallback,
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing_fast_path() {
        // Confiance élevée + texte court + mode Chat → fast path
        assert_eq!(
            route(0.90, 10, WriteMode::Chat),
            RoutingDecision::RulesOnly
        );
    }

    #[test]
    fn test_routing_low_confidence() {
        // Confiance faible → LLM
        assert_eq!(
            route(0.70, 10, WriteMode::Chat),
            RoutingDecision::RulesAndLlm
        );
    }

    #[test]
    fn test_routing_long_text() {
        // Texte long → LLM même avec confiance élevée
        assert_eq!(
            route(0.90, 50, WriteMode::Chat),
            RoutingDecision::RulesAndLlm
        );
    }

    #[test]
    fn test_routing_pro_mode_always_llm() {
        // Pro mode → LLM toujours
        assert_eq!(
            route(0.99, 5, WriteMode::Pro),
            RoutingDecision::RulesAndLlm
        );
    }

    #[test]
    fn test_process_without_llm() {
        // Sans LLM disponible → règles seules
        let result = process("euh je veux partir", 0.90, WriteMode::Chat, None);
        assert!(result.rules_only);
        assert!(!result.text.contains("euh"));
        assert!(result.text.starts_with('J') || result.text.starts_with('j'));
    }

    #[test]
    fn test_process_with_llm_fallback() {
        // LLM qui échoue → fallback sur les règles
        let failing_llm = |_text: &str, _mode: WriteMode| -> anyhow::Result<String> {
            Err(anyhow::anyhow!("LLM not available"))
        };
        let result = process("euh test", 0.60, WriteMode::Chat, Some(&failing_llm));
        assert!(result.rules_only); // fallback sur règles
    }
}
