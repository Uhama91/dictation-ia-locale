/// Modes d'écriture et prompts LLM associés (ADR-009, Task 11, Story 8.1)
///
/// 3 modes définis (ADR de la spec fonctionnelle) :
/// - Chat : correction minimale, ton conservé, structuration si marqueurs
/// - Pro  : reformulation concise, style email/document, structuration agressive
/// - Code : jargon technique préservé, symboles intacts, structuration basique
///
/// Story 8.1 : les prompts intègrent la structuration (listes, paragraphes)
/// via un `StructureHint` passé en paramètre. Le ton diffère par mode,
/// la structure est universelle.

use serde::{Deserialize, Serialize};

use crate::pipeline::rules::StructureHint;

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
    /// Le prompt intègre la structuration (Story 8.1) : le `StructureHint`
    /// ajoute des instructions de format au prompt de base du mode.
    /// Chaque prompt ≤ 60 tokens, avec micro-exemple few-shot pour les listes
    /// (recommandation Gemini : qwen2.5:0.5b réagit mieux aux exemples).
    pub fn system_prompt(&self, hint: StructureHint) -> String {
        let base = match self {
            WriteMode::Chat => {
                "Tu es un correcteur de transcription vocale française. \
                Corrige l'orthographe et la ponctuation. Conserve le ton oral."
            }
            WriteMode::Pro => {
                "Tu es un rédacteur professionnel. \
                Reformule en français formel, concis, adapté pour un email."
            }
            WriteMode::Code => {
                "Tu es un assistant technique. \
                Corrige la ponctuation. Préserve TOUS les termes techniques anglais et symboles. \
                Ne traduis jamais le jargon."
            }
        };

        let structure_instruction = match hint {
            StructureHint::List => match self {
                WriteMode::Chat => {
                    " Exemple: \"d'abord le lait ensuite du pain enfin des oeufs\" → \
                    \"- D'abord le lait\\n- Ensuite du pain\\n- Enfin des oeufs\"\
                    \nFormate les énumérations en liste à tirets (-)."
                }
                WriteMode::Pro => {
                    " Exemple: \"d'abord le lait ensuite du pain enfin des oeufs\" → \
                    \"- Lait\\n- Pain\\n- Oeufs\"\
                    \nListes à tirets, capitalisées."
                }
                WriteMode::Code => {
                    " Si énumération explicite, formate en liste à tirets (-)."
                }
            },
            StructureHint::MultiParagraph => {
                " Si le texte change de sujet, crée des paragraphes séparés."
            }
            StructureHint::Paragraph | StructureHint::SingleMessage => "",
        };

        let suffix = " Réponds uniquement avec le texte corrigé.";

        format!("{}{}{}", base, structure_instruction, suffix)
    }

    /// Indique si ce mode justifie toujours le passage par le LLM
    /// (même avec un score de confiance élevé).
    ///
    /// Story 8.1 : `StructureHint` non-trivial force le LLM dans tous les modes.
    pub fn needs_llm(&self, hint: StructureHint) -> bool {
        match hint {
            StructureHint::List | StructureHint::MultiParagraph => true,
            _ => matches!(self, WriteMode::Pro),
        }
    }

    /// Ancien API conservée pour compatibilité (TODO: supprimer après migration complète)
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
            for hint in [StructureHint::SingleMessage, StructureHint::Paragraph, StructureHint::List, StructureHint::MultiParagraph] {
                assert!(!mode.system_prompt(hint).is_empty());
            }
        }
    }

    #[test]
    fn test_pro_always_needs_llm() {
        assert!(WriteMode::Pro.needs_llm(StructureHint::SingleMessage));
        assert!(WriteMode::Pro.needs_llm(StructureHint::Paragraph));
        assert!(WriteMode::Pro.needs_llm(StructureHint::List));
    }

    #[test]
    fn test_chat_needs_llm_only_for_structure() {
        assert!(!WriteMode::Chat.needs_llm(StructureHint::SingleMessage));
        assert!(!WriteMode::Chat.needs_llm(StructureHint::Paragraph));
        assert!(WriteMode::Chat.needs_llm(StructureHint::List));
        assert!(WriteMode::Chat.needs_llm(StructureHint::MultiParagraph));
    }

    #[test]
    fn test_code_needs_llm_only_for_structure() {
        assert!(!WriteMode::Code.needs_llm(StructureHint::SingleMessage));
        assert!(WriteMode::Code.needs_llm(StructureHint::List));
    }

    #[test]
    fn test_list_prompt_contains_example() {
        let prompt = WriteMode::Chat.system_prompt(StructureHint::List);
        assert!(prompt.contains("tirets"), "Chat+List prompt should mention tirets: {}", prompt);
        assert!(prompt.contains("Exemple"), "Chat+List prompt should contain few-shot example: {}", prompt);
    }

    #[test]
    fn test_single_message_prompt_no_structure_instruction() {
        let prompt = WriteMode::Chat.system_prompt(StructureHint::SingleMessage);
        assert!(!prompt.contains("tirets"), "SingleMessage prompt should not mention lists: {}", prompt);
        assert!(!prompt.contains("Exemple"), "SingleMessage prompt should not contain example: {}", prompt);
    }

    #[test]
    fn test_backward_compat_always_use_llm() {
        assert!(WriteMode::Pro.always_use_llm());
        assert!(!WriteMode::Chat.always_use_llm());
        assert!(!WriteMode::Code.always_use_llm());
    }
}
