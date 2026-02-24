/// Règles de nettoyage local FR — < 1ms, zéro dépendance LLM
///
/// Task 9  : Implémentation complète des règles FR
/// Task 20 : Filler words FR étendus
/// Task 21 : Normalisation élisions (apostrophe-espace Whisper)
/// Task 22 : Suppression ponctuation doublée (.., ??, !!, ...)
///
/// Les regex sont compilées une seule fois via once_cell::sync::Lazy.
/// Note : la crate `regex` ne supporte pas les backreférences — la
/// déduplication de ponctuation est gérée par des regex indépendantes.
use once_cell::sync::Lazy;
use regex::Regex;

/// Filler words francophone courants à supprimer.
/// Liste étendue (Task 20) avec des expressions multi-mots sans ambiguïté.
/// Les alternatives multi-mots sont listées AVANT les sous-chaînes pour éviter
/// les correspondances partielles (ex: "disons que" avant "disons").
static FILLER_WORDS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(euh+|heu+|bah|bon ben|ben|disons que|disons|du coup|en quelque sorte|si tu veux|en tout cas|tu vois|vous voyez|n'est-ce pas|pas vrai|à vrai dire|en gros|genre|voilà|quoi|en fait|eh bien|hein|pfff?|ah bon|eh|ouais bon|bref)\b[,\s]*"
    ).unwrap()
});

/// Normalise les élisions avec espace parasite qu'insère parfois Whisper.
/// Ex: "j' ai" → "j'ai", "c' est" → "c'est"
/// La regex capture la lettre/groupe avant l'apostrophe et supprime l'espace.
/// (Task 21)
static ELISION_SPACE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(j|c|n|l|d|qu)'\s+").unwrap()
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
    Regex::new(r"([.!?])([A-ZÀ-ÙÂÊÎÔÛa-zà-ùâêîôûäëïöü])").unwrap()
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
    fn test_space_after_exclamation() {
        let result = apply("Super!C'est génial");
        assert!(result.contains("! C") || result.contains("! c"),
            "espace attendu après le !: {:?}", result);
    }
}
