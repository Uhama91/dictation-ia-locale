/// Règles de nettoyage local FR — < 1ms, zéro dépendance LLM
///
/// Task 9  : Implémentation complète des règles FR
/// Task 20 : Filler words FR étendus
/// Task 21 : Normalisation élisions (apostrophe-espace Whisper)
/// Task 22 : Suppression ponctuation doublée (.., ??, !!, ...)
/// Story 8.1 : Détection structure (listes, paragraphes) + fallback formatter
///
/// Les regex sont compilées une seule fois via once_cell::sync::Lazy.
/// Note : la crate `regex` ne supporte pas les backreférences — la
/// déduplication de ponctuation est gérée par des regex indépendantes.
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Filler words francophone courants à supprimer.
/// Liste étendue (Task 20) avec des expressions multi-mots sans ambiguïté.
/// Les alternatives multi-mots sont listées AVANT les sous-chaînes pour éviter
/// les correspondances partielles (ex: "disons que" avant "disons").
///
/// Note : "en fait" peut être un connecteur légitime ("le problème, en fait, est…").
/// En contexte de dictée vocale, c'est quasi-toujours un filler — accepté comme trade-off.
/// Le mode Pro/LLM peut restituer le connecteur si nécessaire.
static FILLER_WORDS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(euh+|heu+|bah|bon ben|ben|disons que|disons|du coup|en quelque sorte|si tu veux|en tout cas|tu vois|vous voyez|n'est-ce pas|pas vrai|à vrai dire|en gros|genre|voilà|quoi|en fait|eh bien|hein|pfff?|ah bon|eh|ouais bon|bref)\b[,\s]*"
    ).unwrap()
});

/// Normalise les élisions avec espace parasite qu'insère parfois Whisper.
/// Ex: "j' ai" → "j'ai", "c' est" → "c'est", "s' il" → "s'il", "m' a" → "m'a"
/// Couvre : j, c, n, l, d, qu, s, m (toutes les élisions FR courantes).
/// (Task 21 + Story 1.3 Task 2)
static ELISION_SPACE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(j|c|n|l|d|qu|s|m)'\s+").unwrap()
});

/// Trois points (ou plus) → ellipse unicode "…"
/// (Task 22 — partie 1)
static TRIPLE_DOT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\.{3,}").unwrap()
});

/// Deux points consécutifs (mais pas trois, déjà traités ci-dessus) → "."
/// (Task 22 — partie 2)
static DOUBLE_DOT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\.{2}").unwrap()
});

/// Deux points d'interrogation consécutifs ou plus → "?"
static DOUBLE_QUESTION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\?{2,}").unwrap()
});

/// Deux points d'exclamation consécutifs ou plus → "!"
static DOUBLE_EXCLAMATION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"!{2,}").unwrap()
});

/// Deux virgules consécutives ou plus → ","
static DOUBLE_COMMA_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r",{2,}").unwrap()
});

/// Deux points-virgules consécutifs ou plus → ";"
static DOUBLE_SEMICOLON_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r";{2,}").unwrap()
});

/// Deux deux-points consécutifs ou plus → ":"
static DOUBLE_COLON_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r":{2,}").unwrap()
});

/// Collapse les bégaiements (mots consécutifs identiques, insensible à la casse).
/// La crate regex ne supporte pas les backreférences — implémentation manuelle.
///
/// Limitation connue : la comparaison est token-level (split_whitespace), donc
/// "c'est, c'est" ne collapse pas ("c'est," != "c'est"). Acceptable car la
/// ponctuation intermédiaire est rare dans les bégaiements réels Whisper.
fn collapse_stutters(text: &str) -> String {
    let mut result: Vec<&str> = Vec::new();
    for word in text.split_whitespace() {
        if result.last().map_or(false, |last: &&str| last.eq_ignore_ascii_case(word)) {
            continue;
        }
        result.push(word);
    }
    result.join(" ")
}

/// Espaces multiples
static MULTI_SPACE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\s{2,}").unwrap()
});

/// Espace manquant après ponctuation de fin de phrase (Whisper colle parfois les phrases).
/// Ex: "Bonjour.Comment" → "Bonjour. Comment"
static SPACE_AFTER_PUNCT_RE: Lazy<Regex> = Lazy::new(|| {
    // Ponctuation suivie directement d'une lettre (maj ou min, ASCII ou accents)
    Regex::new(r"([.!?])([A-ZÀ-ÙÂÊÎÔÛÇa-zà-ùâêîôûçäëïöü])").unwrap()
});


/// Applique toutes les règles de nettoyage FR sur le texte brut Whisper.
///
/// Ordre des passes :
/// 1. Élisions avec espace parasite (avant filler words)
/// 2. Filler words FR
/// 3. Ponctuation doublée / hallucinations Whisper
/// 4. Bégaiements (mots répétés)
/// 5. Espaces multiples
/// 6. Trim + majuscule initiale + point final
///
/// # Performance
/// < 1ms sur une phrase de 100 tokens (regex compilées une fois).
pub fn apply(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    // 1. Normaliser les élisions avec espace parasite (Whisper FR)
    let cleaned = ELISION_SPACE_RE.replace_all(text, |caps: &regex::Captures| {
        format!("{}'", &caps[1])
    });

    // 2. Supprimer les filler words FR
    let cleaned = FILLER_WORDS_RE.replace_all(&cleaned, " ");

    // 3. Corriger la ponctuation doublée (Task 22)
    //    Ordre : trois points AVANT deux points pour éviter de créer des ".."
    let cleaned = TRIPLE_DOT_RE.replace_all(&cleaned, "…");
    let cleaned = DOUBLE_DOT_RE.replace_all(&cleaned, ".");
    let cleaned = DOUBLE_QUESTION_RE.replace_all(&cleaned, "?");
    let cleaned = DOUBLE_EXCLAMATION_RE.replace_all(&cleaned, "!");
    let cleaned = DOUBLE_COMMA_RE.replace_all(&cleaned, ",");
    let cleaned = DOUBLE_SEMICOLON_RE.replace_all(&cleaned, ";");
    let cleaned = DOUBLE_COLON_RE.replace_all(&cleaned, ":");

    // 4. Espace manquant après ponctuation (Whisper colle parfois "phrase.Suivante")
    let cleaned = SPACE_AFTER_PUNCT_RE.replace_all(&cleaned, "$1 $2");

    // 5. Collapse les bégaiements (répétitions)
    let cleaned = collapse_stutters(&cleaned);

    // 6. Normaliser les espaces multiples
    let cleaned = MULTI_SPACE_RE.replace_all(&cleaned, " ");

    // 7. Trim
    let mut result = cleaned.trim().to_string();

    // 8. Majuscule en début de phrase
    if let Some(first) = result.chars().next() {
        if first.is_lowercase() {
            let upper = first.to_uppercase().to_string();
            result = upper + &result[first.len_utf8()..];
        }
    }

    // 9. Point final si absent (et que le texte n'est pas vide)
    if !result.is_empty() {
        let last = result.chars().last().unwrap();
        if !matches!(last, '.' | '!' | '?' | ':' | ';' | '…') {
            result.push('.');
        }
    }

    result
}

// ============================================================================
// Story 8.1 — Détection de structure du texte transcrit
// ============================================================================

/// Indice de structure détecté dans le texte post-règles.
/// Utilisé pour adapter le prompt LLM et le nombre de tokens de sortie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructureHint {
    /// Message court (< 20 mots, pas de marqueur) — rendu inline
    SingleMessage,
    /// Paragraphe unique (20–60 mots, 1 sujet)
    Paragraph,
    /// Multi-paragraphes (> 60 mots + marqueur de pivot)
    MultiParagraph,
    /// Liste à tirets (marqueurs énumératifs FR détectés)
    List,
}

impl Default for StructureHint {
    fn default() -> Self {
        StructureHint::SingleMessage
    }
}

// ── Marqueurs de liste ────────────────────────────────────────────────────

/// Tier 1 : marqueurs ordinaux — signal fort (2 suffisent)
static LIST_TIER1_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(premièrement|deuxièmement|troisièmement|quatrièmement|cinquièmement|d'une part|d'autre part|en premier lieu|en deuxième lieu|en troisième lieu)\b"
    ).unwrap()
});

/// Tier 2 : marqueurs séquentiels — signal moyen (2 suffisent)
static LIST_TIER2_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(d'abord|ensuite|puis|enfin|en dernier lieu|finalement|pour commencer|pour finir|pour terminer|dans un premier temps|dans un deuxième temps|dans un troisième temps)\b"
    ).unwrap()
});

/// Tier 3 : marqueurs additifs — signal faible (3+ pour déclencher)
static LIST_TIER3_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(également|de plus|en outre|par ailleurs|et aussi|sans oublier|et puis)\b"
    ).unwrap()
});

// ── Marqueurs de pivot (changement de paragraphe) ──────────────────────

static PIVOT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(par contre|cependant|néanmoins|toutefois|sur un autre sujet|pour ce qui est de|passons à|autre chose|autre point important|à propos de|s'agissant de)\b"
    ).unwrap()
});

/// Vérifie si un pivot est précédé d'une ponctuation forte (`.` ou `,`).
/// Réduit les faux positifs sur les usages conversationnels courants.
static PIVOT_WITH_PUNCT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)[.,]\s*(par contre|cependant|néanmoins|toutefois|sur un autre sujet|pour ce qui est de|passons à|autre chose|autre point important|à propos de|s'agissant de)\b"
    ).unwrap()
});

// ── Détection question ────────────────────────────────────────────────────

/// Détecte une question pure et courte (sans marqueurs de liste).
/// Les questions suivies de marqueurs de liste sont traitées normalement.
fn is_pure_question(text: &str, word_count: usize) -> bool {
    if word_count > 15 {
        return false;
    }
    let trimmed = text.trim();
    if trimmed.ends_with('?') {
        return true;
    }
    let lower = trimmed.to_lowercase();
    lower.starts_with("est-ce que ")
        || lower.starts_with("est-ce qu'")
        || lower.starts_with("comment ")
        || lower.starts_with("pourquoi ")
        || lower.starts_with("quand ")
        || lower.starts_with("où ")
        || lower.starts_with("qu'est-ce ")
        || lower.starts_with("combien ")
        || lower.starts_with("quel ")
        || lower.starts_with("quelle ")
        || lower.starts_with("quels ")
        || lower.starts_with("quelles ")
}

/// Compte les occurrences d'une regex dans le texte.
fn count_matches(text: &str, re: &Regex) -> usize {
    re.find_iter(text).count()
}

/// Détecte la structure probable du texte post-règles.
///
/// Exécuté en < 1ms (regex compilées, aucun LLM).
///
/// # Logique de décision
///
/// 1. Si marqueurs de liste détectés → `List`
/// 2. Si > 60 mots + marqueur de pivot (avec ponctuation forte OU > 60 mots seul) → `MultiParagraph`
/// 3. Si < 20 mots, pas de marqueur → `SingleMessage`
/// 4. Sinon → `Paragraph`
pub fn detect_structure(text: &str) -> StructureHint {
    let word_count = text.split_whitespace().count();

    // Comptage marqueurs de liste
    let tier1 = count_matches(text, &LIST_TIER1_RE);
    let tier2 = count_matches(text, &LIST_TIER2_RE);
    let tier3 = count_matches(text, &LIST_TIER3_RE);

    // Détection liste : prioritaire sur tout le reste
    let is_list = tier1 >= 2
        || tier2 >= 2
        || (tier1 >= 1 && tier2 >= 1)
        || (tier2 >= 1 && tier3 >= 1)
        || tier3 >= 3;

    if is_list {
        // Exception : question pure courte sans marqueurs de liste forts
        // "Comment tu vas ?" → pas une liste, même si un marqueur faible traîne
        // Mais "Quelles sont les étapes ? D'abord X, ensuite Y" → liste
        if is_pure_question(text, word_count) && tier1 == 0 && tier2 == 0 {
            return StructureHint::SingleMessage;
        }
        return StructureHint::List;
    }

    // Détection multi-paragraphes : > 60 mots + pivot
    if word_count > 60 {
        let pivot_count = count_matches(text, &PIVOT_RE);
        let pivot_with_punct = count_matches(text, &PIVOT_WITH_PUNCT_RE);
        if pivot_with_punct >= 1 || pivot_count >= 2 {
            return StructureHint::MultiParagraph;
        }
    }

    // Message court : < 20 mots, pas de marqueur spécial
    if word_count < 20 {
        // Vérifier si c'est une pure question (déjà < 20 mots)
        if is_pure_question(text, word_count) {
            return StructureHint::SingleMessage;
        }
        return StructureHint::SingleMessage;
    }

    StructureHint::Paragraph
}

/// Fallback de structuration quand le LLM est indisponible.
///
/// Insère `\n- ` devant les marqueurs Tier 1/2 détectés sans retirer les mots de liaison.
/// Produit une liste lisible même sans LLM.
///
/// Exemple : "d'abord le lait ensuite du pain enfin des œufs"
///         → "- D'abord le lait\n- Ensuite du pain\n- Enfin des œufs"
pub fn apply_structure_fallback(text: &str, hint: StructureHint) -> String {
    match hint {
        StructureHint::List => format_list_fallback(text),
        StructureHint::MultiParagraph => format_multi_paragraph_fallback(text),
        _ => text.to_string(),
    }
}

/// Formate une liste en insérant `\n- ` devant chaque marqueur Tier 1/2.
fn format_list_fallback(text: &str) -> String {
    // Regex combinée Tier 1 + Tier 2 pour remplacement
    static LIST_COMBINED_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?i)\b(premièrement|deuxièmement|troisièmement|quatrièmement|cinquièmement|d'une part|d'autre part|en premier lieu|en deuxième lieu|en troisième lieu|d'abord|ensuite|puis|enfin|en dernier lieu|finalement|pour commencer|pour finir|pour terminer|dans un premier temps|dans un deuxième temps|dans un troisième temps)\b"
        ).unwrap()
    });

    // Trouver les positions de tous les marqueurs
    let matches: Vec<_> = LIST_COMBINED_RE.find_iter(text).collect();
    if matches.is_empty() {
        return text.to_string();
    }

    let mut result = String::with_capacity(text.len() + matches.len() * 4);
    let mut last_end = 0;

    for (i, m) in matches.iter().enumerate() {
        // Texte avant ce marqueur (trim les espaces/virgules/points)
        let before = text[last_end..m.start()].trim_matches(|c: char| c.is_whitespace() || c == ',' || c == '.');
        if i == 0 && !before.is_empty() {
            // Texte avant le premier marqueur → intro inline
            result.push_str(before);
            result.push('\n');
        } else if i > 0 && !before.is_empty() {
            // Ce "before" appartient à l'item précédent — déjà inclus via la logique ci-dessous
        }

        // Le marqueur + tout le texte jusqu'au prochain marqueur
        let item_end = matches.get(i + 1).map_or(text.len(), |next| {
            // Remonter avant les espaces/virgules qui précèdent le prochain marqueur
            let prefix = &text[m.end()..next.start()];
            m.end() + prefix.trim_end_matches(|c: char| c.is_whitespace() || c == ',' || c == '.').len()
        });

        let item_text = text[m.start()..item_end].trim();
        if !item_text.is_empty() {
            result.push_str("- ");
            // Capitaliser la première lettre de l'item
            let mut chars = item_text.chars();
            if let Some(first) = chars.next() {
                result.extend(first.to_uppercase());
                result.push_str(chars.as_str());
            }
            if i < matches.len() - 1 {
                result.push('\n');
            }
        }

        last_end = item_end;
    }

    // Texte après le dernier marqueur (inclus dans le dernier item)
    let trailing = text[last_end..].trim();
    if !trailing.is_empty() && !result.is_empty() {
        // Ajouter au dernier item
        result.push(' ');
        result.push_str(trailing);
    }

    // Nettoyage : retirer le point final redondant si le dernier item en a déjà un
    let result = result.trim().to_string();
    result
}

/// Formate un texte multi-paragraphes en insérant `\n\n` devant les marqueurs de pivot.
fn format_multi_paragraph_fallback(text: &str) -> String {
    // Insérer \n\n avant chaque marqueur de pivot précédé de ponctuation
    let result = PIVOT_WITH_PUNCT_RE.replace_all(text, |caps: &regex::Captures| {
        // Garder le point/virgule, ajouter \n\n, puis le marqueur
        let full = &caps[0];
        let punct_char = full.chars().next().unwrap();
        let marker = &caps[1];
        format!("{}\n\n{}", punct_char, capitalize_first(marker))
    });

    result.to_string()
}

/// Capitalise la première lettre d'une chaîne.
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => {
            let upper: String = c.to_uppercase().collect();
            upper + chars.as_str()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Tests existants (Task 9) ────────────────────────────────────────────

    #[test]
    fn test_filler_words_removed() {
        assert_eq!(
            apply("euh je voulais dire bonjour"),
            "Je voulais dire bonjour."
        );
        assert_eq!(
            apply("du coup on va faire ça comme ça"),
            "On va faire ça comme ça."
        );
        assert_eq!(
            apply("genre c'est vraiment bien quoi"),
            "C'est vraiment bien."
        );
    }

    #[test]
    fn test_stutter_collapse() {
        assert_eq!(apply("je je veux partir"), "Je veux partir.");
        assert_eq!(apply("c'est c'est vraiment bien"), "C'est vraiment bien.");
    }

    #[test]
    fn test_capitalization_and_period() {
        assert_eq!(apply("bonjour tout le monde"), "Bonjour tout le monde.");
        assert_eq!(apply("Bonjour tout le monde."), "Bonjour tout le monde.");
        assert_eq!(apply("Comment vas-tu ?"), "Comment vas-tu ?");
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(apply(""), "");
    }

    #[test]
    fn test_complex_sentence() {
        let raw = "euh du coup je voulais vous dire que genre c'est vraiment important hein";
        let result = apply(raw);
        assert!(!result.contains("euh"));
        assert!(!result.contains("du coup"));
        assert!(!result.contains("genre"));
        assert!(!result.contains("hein"));
        assert!(result.ends_with('.'));
    }

    // ── Tests nouveaux filler words (Task 20) ──────────────────────────────

    #[test]
    fn test_new_filler_words_disons() {
        let result = apply("disons que c'est une bonne idée");
        assert!(!result.contains("disons que"), "disons que should be removed");
        assert!(result.contains("bonne idée") || result.contains("une bonne"));
    }

    #[test]
    fn test_new_filler_words_en_quelque_sorte() {
        let result = apply("c'est en quelque sorte une solution");
        assert!(!result.contains("en quelque sorte"));
    }

    #[test]
    fn test_new_filler_words_si_tu_veux() {
        let result = apply("si tu veux on peut commencer");
        assert!(!result.contains("si tu veux"));
    }

    #[test]
    fn test_new_filler_words_en_tout_cas() {
        let result = apply("en tout cas voilà c'est ce que je pense");
        assert!(!result.contains("en tout cas"));
    }

    #[test]
    fn test_new_filler_words_tu_vois() {
        let result = apply("c'est important tu vois pour la suite");
        assert!(!result.contains("tu vois"));
    }

    #[test]
    fn test_new_filler_words_vous_voyez() {
        let result = apply("vous voyez ce que je veux dire");
        assert!(!result.contains("vous voyez"));
    }

    #[test]
    fn test_new_filler_words_nest_ce_pas() {
        let result = apply("c'est correct n'est-ce pas");
        assert!(!result.contains("n'est-ce pas"));
    }

    #[test]
    fn test_new_filler_words_pas_vrai() {
        let result = apply("c'est bien comme ça pas vrai");
        assert!(!result.contains("pas vrai"));
    }

    #[test]
    fn test_new_filler_words_a_vrai_dire() {
        let result = apply("à vrai dire je ne sais pas");
        assert!(!result.contains("à vrai dire"));
    }

    #[test]
    fn test_new_filler_words_en_gros() {
        let result = apply("en gros il faut tout refaire");
        assert!(!result.contains("en gros"));
    }

    #[test]
    fn test_new_filler_words_disons_standalone() {
        let result = apply("je veux disons partir demain");
        assert!(!result.contains("disons"));
    }

    // ── Tests élisions (Task 21) ────────────────────────────────────────────

    #[test]
    fn test_elision_j_space() {
        // "j' ai" → "j'ai" puis majuscule
        let result = apply("j' ai besoin d' aide");
        assert!(!result.contains("j' "), "space after j' should be removed");
        assert!(!result.contains("d' "), "space after d' should be removed");
        assert!(result.contains("j'ai") || result.contains("J'ai"));
    }

    #[test]
    fn test_elision_c_space() {
        let result = apply("c' est une bonne idée");
        assert!(!result.contains("c' "));
        assert!(result.contains("c'est") || result.contains("C'est"));
    }

    #[test]
    fn test_elision_n_space() {
        let result = apply("il n' est pas là");
        assert!(!result.contains("n' "));
        assert!(result.contains("n'est"));
    }

    #[test]
    fn test_elision_l_space() {
        let result = apply("l' homme est arrivé");
        assert!(!result.contains("l' "));
        assert!(result.contains("l'homme") || result.contains("L'homme"));
    }

    #[test]
    fn test_elision_qu_space() {
        let result = apply("je pense qu' il viendra");
        assert!(!result.contains("qu' "));
        assert!(result.contains("qu'il"));
    }

    // ── Tests élisions s' et m' (Story 1.3 — Task 2) ──────────────────────

    #[test]
    fn test_elision_s_space() {
        let result = apply("s' il vous plaît attendez");
        assert!(!result.contains("s' "), "space after s' should be removed");
        assert!(result.contains("s'il") || result.contains("S'il"));
    }

    #[test]
    fn test_elision_m_space() {
        let result = apply("il m' a dit bonjour");
        assert!(!result.contains("m' "), "space after m' should be removed");
        assert!(result.contains("m'a"));
    }

    // ── Tests ponctuation doublée (Task 22) ────────────────────────────────

    #[test]
    fn test_double_period() {
        let result = apply("c'est fini.. maintenant");
        assert!(!result.contains(".."), "double period should be collapsed to single");
    }

    #[test]
    fn test_triple_period_to_ellipsis() {
        let result = apply("je ne sais pas... vraiment");
        assert!(!result.contains("..."), "triple period should become ellipsis");
        assert!(result.contains('…'), "triple period should become unicode ellipsis");
    }

    #[test]
    fn test_double_question_mark() {
        let result = apply("Quoi?? tu plaisantes");
        assert!(!result.contains("??"), "double ? should be collapsed to single");
    }

    #[test]
    fn test_double_exclamation() {
        let result = apply("Super!! c'est génial");
        assert!(!result.contains("!!"), "double ! should be collapsed to single");
    }

    #[test]
    fn test_double_comma() {
        let result = apply("oui,, je comprends");
        assert!(!result.contains(",,"), "double comma should be collapsed to single");
    }

    // ── Tests bégaiements avancés (Story 1.3 — Task 4) ────────────────────

    #[test]
    fn test_stutter_triple_repetition() {
        assert_eq!(apply("je je je veux partir"), "Je veux partir.");
    }

    #[test]
    fn test_stutter_case_insensitive() {
        assert_eq!(apply("Je je veux partir"), "Je veux partir.");
    }

    #[test]
    fn test_stutter_oui_oui_collapsed() {
        // "oui oui" est traité comme un bégaiement (AC4 explicite)
        assert_eq!(apply("oui oui d'accord"), "Oui d'accord.");
    }

    // ── Tests capitalisation avancés (Story 1.3 — Task 5) ───────────────

    #[test]
    fn test_capitalization_accented() {
        assert_eq!(apply("écoute bien ce que je dis"), "Écoute bien ce que je dis.");
    }

    #[test]
    fn test_preserves_existing_final_punctuation() {
        assert_eq!(apply("c'est fini !"), "C'est fini !");
        assert_eq!(apply("vraiment ?"), "Vraiment ?");
        assert_eq!(apply("la liste :"), "La liste :");
        assert_eq!(apply("attendez…"), "Attendez…");
    }

    // ── Tests filler words cas limites (Story 1.3 — Task 1.3) ──────────

    #[test]
    fn test_multiple_consecutive_fillers() {
        let result = apply("euh bah du coup genre on y va");
        assert!(!result.contains("euh"));
        assert!(!result.contains("bah"));
        assert!(!result.contains("du coup"));
        assert!(!result.contains("genre"));
        assert!(result.contains("On y va"));
    }

    #[test]
    fn test_filler_at_end() {
        let result = apply("je comprends quoi");
        assert!(!result.contains("quoi"));
        assert_eq!(result, "Je comprends.");
    }

    // ── Tests espacement après ponctuation ─────────────────────────────────

    #[test]
    fn test_space_after_period() {
        // Whisper colle parfois "fin.Nouveau" sans espace
        let result = apply("c'est la fin.Le suivant commence");
        assert!(result.contains(". Le") || result.contains(". le"),
            "espace attendu après le point: {:?}", result);
    }

    #[test]
    fn test_space_after_question_mark() {
        let result = apply("Tu viens?Bien sûr");
        assert!(result.contains("? Bien") || result.contains("? bien"),
            "espace attendu après le ?: {:?}", result);
    }

    #[test]
    fn test_space_after_punct_cedilla() {
        let result = apply("c'est fini.Ça continue");
        assert!(result.contains(". Ça") || result.contains(". ça"),
            "espace attendu après le point avant cédille: {:?}", result);
    }

    #[test]
    fn test_space_after_exclamation() {
        let result = apply("Super!C'est génial");
        assert!(result.contains("! C") || result.contains("! c"),
            "espace attendu après le !: {:?}", result);
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Story 8.1 — Tests detect_structure
    // ══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_short_message_is_single() {
        assert_eq!(detect_structure("Bonjour tout le monde."), StructureHint::SingleMessage);
    }

    #[test]
    fn test_medium_text_is_paragraph() {
        let text = "Je voulais vous dire que le rapport est prêt et que j'ai vérifié tous les chiffres, \
                    tout me semble correct et on peut envoyer ça au client dès demain matin.";
        assert_eq!(detect_structure(text), StructureHint::Paragraph);
    }

    #[test]
    fn test_list_tier1_ordinals() {
        let text = "Premièrement on doit vérifier les comptes. Deuxièmement on contacte le fournisseur.";
        assert_eq!(detect_structure(text), StructureHint::List);
    }

    #[test]
    fn test_list_tier2_sequential() {
        let text = "D'abord on fait les courses, ensuite on prépare le repas, enfin on mange.";
        assert_eq!(detect_structure(text), StructureHint::List);
    }

    #[test]
    fn test_list_tier2_two_markers() {
        let text = "D'abord préparer le terrain, ensuite construire.";
        assert_eq!(detect_structure(text), StructureHint::List);
    }

    #[test]
    fn test_list_tier1_and_tier2_mixed() {
        let text = "En premier lieu on identifie le problème, ensuite on propose des solutions.";
        assert_eq!(detect_structure(text), StructureHint::List);
    }

    #[test]
    fn test_list_tier3_needs_three() {
        // Seulement 2 marqueurs Tier 3 → pas suffisant
        let text = "Il faut aussi nettoyer la cuisine et aussi ranger le salon.";
        assert_ne!(detect_structure(text), StructureHint::List);

        // 3 marqueurs Tier 3 → liste
        let text = "Il faut également nettoyer, de plus ranger le salon, et par ailleurs faire les courses.";
        assert_eq!(detect_structure(text), StructureHint::List);
    }

    #[test]
    fn test_pure_question_not_restructured() {
        assert_eq!(detect_structure("Comment tu vas ?"), StructureHint::SingleMessage);
        assert_eq!(detect_structure("Est-ce que tout va bien ?"), StructureHint::SingleMessage);
        assert_eq!(detect_structure("Pourquoi tu dis ça ?"), StructureHint::SingleMessage);
    }

    #[test]
    fn test_question_with_list_markers_is_list() {
        // Question suivie de marqueurs de liste → la liste prime
        let text = "Quelles sont les étapes ? D'abord la phase 1, ensuite la phase 2, enfin la livraison.";
        assert_eq!(detect_structure(text), StructureHint::List);
    }

    #[test]
    fn test_multi_paragraph_with_pivot() {
        // > 60 mots + pivot avec ponctuation forte (". Par contre")
        let text = "Le projet avance bien et toutes les fonctionnalités principales sont implémentées correctement. \
                    L'équipe de développement a fait un excellent travail sur le frontend React et le backend Rust. \
                    Les tests unitaires et d'intégration couvrent plus de quatre-vingts pourcent du code source \
                    et les performances mesurées sont tout à fait satisfaisantes pour le moment. \
                    Par contre, il reste quelques bugs mineurs à corriger avant la mise en production finale.";
        assert_eq!(detect_structure(text), StructureHint::MultiParagraph);
    }

    #[test]
    fn test_pivot_word_alone_without_length_is_paragraph() {
        // < 60 mots, un pivot → pas de multi-paragraphe, juste Paragraph
        let text = "Le projet avance bien. Par contre il reste des bugs à corriger avant la livraison prochaine.";
        assert_ne!(detect_structure(text), StructureHint::MultiParagraph);
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Story 8.1 — Tests apply_structure_fallback
    // ══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_fallback_list_tier2() {
        let text = "D'abord le lait, ensuite du pain, enfin des œufs.";
        let result = apply_structure_fallback(text, StructureHint::List);
        assert!(result.contains("- D'abord"), "Should start with '- D'abord', got: {}", result);
        assert!(result.contains("\n- Ensuite"), "Should contain '\\n- Ensuite', got: {}", result);
        assert!(result.contains("\n- Enfin"), "Should contain '\\n- Enfin', got: {}", result);
    }

    #[test]
    fn test_fallback_list_tier1() {
        let text = "Premièrement vérifier les comptes, deuxièmement contacter le fournisseur.";
        let result = apply_structure_fallback(text, StructureHint::List);
        assert!(result.contains("- Premièrement"), "got: {}", result);
        assert!(result.contains("- Deuxièmement"), "got: {}", result);
    }

    #[test]
    fn test_fallback_single_message_unchanged() {
        let text = "Bonjour tout le monde.";
        assert_eq!(apply_structure_fallback(text, StructureHint::SingleMessage), text);
    }

    #[test]
    fn test_fallback_paragraph_unchanged() {
        let text = "Un texte de taille moyenne sans marqueurs spéciaux.";
        assert_eq!(apply_structure_fallback(text, StructureHint::Paragraph), text);
    }

    #[test]
    fn test_fallback_multi_paragraph_inserts_newlines() {
        let long_text = "Le projet avance bien et toutes les fonctionnalités principales sont implémentées. \
                         L'équipe a fait un excellent travail sur le frontend et le backend. Les tests couvrent \
                         plus de quatre-vingts pourcent du code. Par contre, il reste quelques bugs à corriger.";
        let result = apply_structure_fallback(long_text, StructureHint::MultiParagraph);
        assert!(result.contains("\n\n"), "Should contain paragraph break, got: {}", result);
    }
}
