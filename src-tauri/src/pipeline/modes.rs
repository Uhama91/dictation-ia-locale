/// Modes d'écriture et prompts LLM associés (ADR-009, Task 11)
///
/// 3 modes définis (ADR de la spec fonctionnelle) :
/// - Chat : correction minimale, ton conservé
/// - Pro  : reformulation concise, style email/document
/// - Code : jargon technique préservé, symboles intacts

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WriteMode {
    #[default]
    Chat,
    Pro,
    Code,
}

impl WriteMode {
    /// Retourne le prompt système à injecter dans le LLM de nettoyage.
    ///
    /// Le prompt est conçu pour être minimal (< 50 tokens) et directif.
    /// Retourne uniquement le texte nettoyé, jamais d'explication.
    pub fn system_prompt(&self) -> &'static str {
        match self {
            WriteMode::Chat => {
                "Tu es un correcteur de transcription vocale. \
                Corrige uniquement l'orthographe évidente et ajoute la ponctuation de base. \
                Conserve exactement le ton et la structure originale. \
                Ne reformule pas. Réponds uniquement avec le texte corrigé."
            }
            WriteMode::Pro => {
                "Tu es un rédacteur professionnel. \
                Reformule ce texte transcrit de manière concise et professionnelle, \
                adapté pour un email ou document. Paragraphes clairs, ton poli mais direct. \
                Réponds uniquement avec le texte reformulé."
            }
            WriteMode::Code => {
                "Tu es un assistant technique. \
                Corrige la ponctuation de ce texte transcrit. \
                Préserve TOUS les termes techniques anglais, identifiants, symboles et noms de variables. \
                Ne traduis jamais le jargon technique. \
                Réponds uniquement avec le texte corrigé."
            }
        }
    }

    /// Indique si ce mode justifie toujours le passage par le LLM
    /// (même avec un score de confiance élevé)
    pub fn always_use_llm(&self) -> bool {
        matches!(self, WriteMode::Pro)
    }
}

impl std::fmt::Display for WriteMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteMode::Chat => write!(f, "chat"),
            WriteMode::Pro => write!(f, "pro"),
            WriteMode::Code => write!(f, "code"),
        }
    }
}

impl std::str::FromStr for WriteMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "chat" => Ok(WriteMode::Chat),
            "pro" => Ok(WriteMode::Pro),
            "code" => Ok(WriteMode::Code),
            _ => Err(format!("Unknown write mode: '{}'. Use chat/pro/code", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_mode_from_str() {
        assert_eq!(WriteMode::from_str("chat").unwrap(), WriteMode::Chat);
        assert_eq!(WriteMode::from_str("PRO").unwrap(), WriteMode::Pro);
        assert_eq!(WriteMode::from_str("Code").unwrap(), WriteMode::Code);
        assert!(WriteMode::from_str("invalid").is_err());
    }

    #[test]
    fn test_system_prompts_not_empty() {
        for mode in [WriteMode::Chat, WriteMode::Pro, WriteMode::Code] {
            assert!(!mode.system_prompt().is_empty());
        }
    }

    #[test]
    fn test_pro_always_uses_llm() {
        assert!(WriteMode::Pro.always_use_llm());
        assert!(!WriteMode::Chat.always_use_llm());
        assert!(!WriteMode::Code.always_use_llm());
    }
}
