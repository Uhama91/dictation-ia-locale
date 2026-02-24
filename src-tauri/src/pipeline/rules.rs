/// Règles de nettoyage local FR — < 1ms, zéro dépendance LLM
///
/// Task 9 : Implémentation complète des règles FR
/// - Filler words FR
/// - Collapse bégaiements
/// - Majuscule début + point final
///
/// Les regex sont compilées une seule fois via lazy_static pour zéro overhead.
use once_cell::sync::Lazy;
use regex::Regex;

/// Filler words francophone courants à supprimer
static FILLER_WORDS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(euh+|heu+|bah|ben|bon ben|du coup|genre|voilà|quoi|en fait|eh bien|hein|pfff?|ah bon|eh|ouais bon|bref)\b[,\s]*"
    ).unwrap()
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

/// Applique toutes les règles de nettoyage FR sur le texte brut Whisper.
///
/// # Performance
/// < 1ms sur une phrase de 100 tokens (regex compilées une fois).
pub fn apply(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    // 1. Supprimer les filler words FR
    let cleaned = FILLER_WORDS_RE.replace_all(text, " ");

    // 2. Collapse les bégaiements (répétitions)
    let cleaned = collapse_stutters(&cleaned);

    // 3. Normaliser les espaces multiples
    let cleaned = MULTI_SPACE_RE.replace_all(&cleaned, " ");

    // 4. Trim
    let mut result = cleaned.trim().to_string();

    // 5. Majuscule en début de phrase
    if let Some(first) = result.chars().next() {
        if first.is_lowercase() {
            let upper = first.to_uppercase().to_string();
            result = upper + &result[first.len_utf8()..];
        }
    }

    // 6. Point final si absent (et que le texte n'est pas vide)
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
}
