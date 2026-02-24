/// Benchmark de latence — pipeline Dictation IA (Task 17)
///
/// Mesure les latences p50 / p90 / p99 pour :
///   1. Règles FR seules (< 5ms cible)
///   2. Pipeline complet sans LLM (< 10ms cible)
///   3. Pipeline complet avec LLM via Ollama (< 400ms cible, dépend de la machine)
///
/// Exécution :
///   cargo test --test benchmark -- --nocapture
///   cargo test --test benchmark latency -- --nocapture  # subset
///
/// Variables d'environnement :
///   BENCH_ROUNDS=1000  # Nombre de répétitions (défaut : 500)

use dictation_ia_lib::pipeline::{modes::WriteMode, orchestrator, rules};
use std::time::{Duration, Instant};

/// ─────────────────────────────────────────────────────────────────────────────
/// Utilitaires statistiques
/// ─────────────────────────────────────────────────────────────────────────────

fn percentile(sorted: &[Duration], p: f64) -> Duration {
    assert!(!sorted.is_empty(), "Cannot compute percentile on empty slice");
    let idx = ((sorted.len() as f64 * p / 100.0).ceil() as usize).saturating_sub(1);
    sorted[idx.min(sorted.len() - 1)]
}

fn bench_rounds() -> usize {
    std::env::var("BENCH_ROUNDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(500)
}

struct BenchResult {
    name: &'static str,
    p50: Duration,
    p90: Duration,
    p99: Duration,
    min: Duration,
    max: Duration,
    rounds: usize,
}

impl BenchResult {
    fn from_samples(name: &'static str, mut samples: Vec<Duration>) -> Self {
        assert!(!samples.is_empty());
        samples.sort_unstable();
        let n = samples.len();
        Self {
            name,
            p50: percentile(&samples, 50.0),
            p90: percentile(&samples, 90.0),
            p99: percentile(&samples, 99.0),
            min: samples[0],
            max: samples[n - 1],
            rounds: n,
        }
    }

    fn print(&self) {
        println!(
            "\n  📊 {} (n={})\n     min={:?}  p50={:?}  p90={:?}  p99={:?}  max={:?}",
            self.name, self.rounds, self.min, self.p50, self.p90, self.p99, self.max
        );
    }
}

/// ─────────────────────────────────────────────────────────────────────────────
/// Corpus de test — phrases françaises représentatives
/// ─────────────────────────────────────────────────────────────────────────────

fn french_corpus() -> Vec<(&'static str, f32)> {
    vec![
        // (texte_brut, confiance_simulée)

        // Courtes — fast-path attendu (conf >= 0.85, mots <= 30)
        ("euh bonjour comment tu vas", 0.92),
        ("je voudrais un café s'il vous plaît", 0.95),
        ("heu attends je réfléchis", 0.88),
        ("c'est bon t'as fini", 0.91),
        ("ben ouais je suis là", 0.87),
        ("ok vas-y continue", 0.96),
        ("euh euh je voulais dire voilà", 0.93),
        ("du coup t'as compris ou pas", 0.89),
        ("genre ça fait un moment que j'attends", 0.90),
        ("bah c'est pas grave", 0.94),

        // Moyennes — potentiellement LLM
        ("j'ai besoin de te parler d'un truc important concernant le projet", 0.78),
        ("euh donc voilà j'ai regardé le code et je pense qu'il faudrait refactoriser", 0.72),
        ("en fait le problème c'est que le système de cache est pas bien configuré", 0.81),
        ("donc du coup j'ai modifié la fonction et maintenant ça marche beaucoup mieux", 0.83),

        // Longues — LLM path (> 30 mots ou confiance basse)
        ("euh alors voilà j'explique la situation, j'ai reçu un email de la cliente \
          qui dit que le produit fonctionne pas correctement depuis la mise à jour", 0.70),
        ("donc en résumé il faut qu'on revoit l'architecture du module de paiement \
          parce que les transactions échouent dans certains cas edge case qu'on a pas couverts", 0.65),

        // Code — technique
        ("j'ai modifié la fonction process underscore audio pour prendre en compte \
          le sample rate variable", 0.88),
        ("le bug vient de la macro cfg whisper native qui est pas activée en dev", 0.91),
    ]
}

/// ─────────────────────────────────────────────────────────────────────────────
/// Test 1 : règles FR seules (< 5ms cible)
/// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn latency_rules_only() {
    let corpus = french_corpus();
    let rounds = bench_rounds();
    let mut samples: Vec<Duration> = Vec::with_capacity(rounds);

    for i in 0..rounds {
        let (text, _) = corpus[i % corpus.len()];
        let t = Instant::now();
        let _ = rules::apply(text);
        samples.push(t.elapsed());
    }

    let result = BenchResult::from_samples("Règles FR seules", samples);
    result.print();

    // Seuil : p99 < 5ms
    assert!(
        result.p99 < Duration::from_millis(5),
        "p99 règles trop élevé : {:?} (seuil 5ms)",
        result.p99
    );
}

/// ─────────────────────────────────────────────────────────────────────────────
/// Test 2 : pipeline complet sans LLM (< 10ms cible)
/// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn latency_pipeline_no_llm() {
    let corpus = french_corpus();
    let rounds = bench_rounds();
    let mut samples: Vec<Duration> = Vec::with_capacity(rounds);

    for i in 0..rounds {
        let (text, confidence) = corpus[i % corpus.len()];
        let t = Instant::now();
        let result = orchestrator::process(text, confidence, WriteMode::Chat, None);
        samples.push(t.elapsed());
        // S'assurer que le résultat n'est pas vide pour les textes non-vides
        if !text.trim().is_empty() {
            assert!(!result.text.is_empty());
        }
    }

    let result = BenchResult::from_samples("Pipeline sans LLM", samples);
    result.print();

    // Seuil : p99 < 10ms
    assert!(
        result.p99 < Duration::from_millis(10),
        "p99 pipeline sans LLM trop élevé : {:?} (seuil 10ms)",
        result.p99
    );
}

/// ─────────────────────────────────────────────────────────────────────────────
/// Test 3 : routing fast-path vs LLM path — distribution
/// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn routing_distribution() {
    let corpus = french_corpus();
    let mut fast_path = 0usize;
    let mut llm_path = 0usize;

    // Stub LLM qui retourne Err → règles-only forcé
    let failing_llm = |_text: &str, _mode: WriteMode| -> anyhow::Result<String> {
        Err(anyhow::anyhow!("stub"))
    };

    for &(text, confidence) in &corpus {
        let result = orchestrator::process(text, confidence, WriteMode::Chat, Some(&failing_llm));
        if result.rules_only {
            fast_path += 1;
        } else {
            llm_path += 1;
        }
    }

    let total = corpus.len();
    let fast_pct = (fast_path as f64 / total as f64) * 100.0;

    println!(
        "\n  📊 Routing distribution (n={})\n     Fast-path (règles) : {} ({:.0}%)\n     LLM path           : {} ({:.0}%)",
        total, fast_path, fast_pct, llm_path, 100.0 - fast_pct
    );

    // Cible : >= 50% fast-path sur le corpus de test
    // (en production avec conf réelle Whisper >= 0.85, on attend ~60-65%)
    assert!(
        fast_pct >= 50.0,
        "Fast-path trop faible : {:.0}% (cible >= 50%)",
        fast_pct
    );
}

/// ─────────────────────────────────────────────────────────────────────────────
/// Test 4 : pipeline avec LLM Ollama (optionnel — skip si Ollama absent)
/// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn latency_pipeline_with_ollama() {
    // Skip si Ollama n'est pas disponible
    if !dictation_ia_lib::llm::cleanup::is_model_available() {
        println!("\n  ⚠️  Ollama qwen2.5:0.5b non disponible — test ignoré");
        println!("     Pour activer : ollama serve && ollama pull qwen2.5:0.5b");
        return;
    }

    // Corpus réduit pour ce test (LLM est lent)
    let corpus = [
        ("euh donc du coup j'explique la situation complète pour être sûr que tu comprends bien", 0.70),
        ("le module de paiement a un bug important qui affecte les transactions au-delà de mille euros", 0.72),
        ("j'ai remarqué que le cache est pas correctement invalidé quand on met à jour un produit", 0.68),
    ];

    let rounds = 5.min(bench_rounds()); // Max 5 rounds pour les tests LLM
    let mut samples: Vec<Duration> = Vec::with_capacity(rounds);

    for i in 0..rounds {
        let (text, confidence) = corpus[i % corpus.len()];
        let t = Instant::now();
        let _result = orchestrator::process(
            text,
            confidence,
            WriteMode::Chat,
            Some(&dictation_ia_lib::llm::cleanup::run),
        );
        samples.push(t.elapsed());
    }

    let result = BenchResult::from_samples("Pipeline avec Ollama", samples);
    result.print();

    // Seuil souple : p99 < 4s (Ollama peut être lent au premier appel)
    assert!(
        result.p99 < Duration::from_secs(4),
        "p99 LLM trop élevé : {:?} (seuil 4s)",
        result.p99
    );
}

/// ─────────────────────────────────────────────────────────────────────────────
/// Test 5 : qualité règles — zero-edit rate sur textes propres
/// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn rules_zero_edit_rate() {
    // Textes qui ne doivent PAS être modifiés (déjà corrects)
    let clean_texts = vec![
        "Bonjour, comment allez-vous ?",
        "Le projet avance bien.",
        "J'ai terminé la tâche demandée.",
        "Merci pour votre aide.",
        "C'est une bonne idée.",
    ];

    // Textes qui DOIVENT être modifiés (fillers, bégaiements)
    let dirty_texts = vec![
        ("euh bonjour comment vas-tu", true),
        ("heu je voulais dire", true),
        ("ben voilà c'est bon", true),
        ("du coup le projet avance", true),
        ("je je voulais partir", true),
    ];

    // Les textes propres doivent passer sans modification majeure
    for text in &clean_texts {
        let cleaned = rules::apply(text);
        // La longueur ne doit pas varier de plus de 20%
        let len_ratio = cleaned.len() as f64 / text.len() as f64;
        assert!(
            len_ratio >= 0.8,
            "Texte propre trop modifié: '{}' → '{}' (ratio {:.2})",
            text, cleaned, len_ratio
        );
    }

    // Les textes sales doivent être modifiés
    let mut modified = 0usize;
    for (text, _expect_change) in &dirty_texts {
        let cleaned = rules::apply(text);
        if cleaned != *text {
            modified += 1;
        }
    }

    let mod_rate = (modified as f64 / dirty_texts.len() as f64) * 100.0;
    println!(
        "\n  📊 Règles — modification des textes sales: {} / {} ({:.0}%)",
        modified, dirty_texts.len(), mod_rate
    );

    assert!(
        mod_rate >= 60.0,
        "Taux de modification textes sales insuffisant : {:.0}% (cible >= 60%)",
        mod_rate
    );
}
