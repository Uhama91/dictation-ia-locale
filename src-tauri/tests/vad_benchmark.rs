/// Benchmark de latence — Silero VAD (Task 7)
///
/// Mesure les latences p50 / p99 pour une trame 30 ms à 16 kHz (480 échantillons).
/// Cibles : p50 < 2 ms, p99 < 10 ms.
///
/// Exécution :
///   cargo test --test vad_benchmark -- --nocapture
///   cargo test --test vad_benchmark vad -- --nocapture

use dictation_ia_lib::audio_toolkit::{vad::SmoothedVad, vad::VoiceActivityDetector, SileroVad};
use std::path::PathBuf;
use std::time::{Duration, Instant};

// ─────────────────────────────────────────────────────────────────────────────
// Constantes
// ─────────────────────────────────────────────────────────────────────────────

/// 30 ms à 16 kHz
const FRAME_SAMPLES: usize = 480;
/// Fréquence du signal de test (Hz)
const SINE_FREQ_HZ: f32 = 440.0;
/// Taux d'échantillonnage (Hz)
const SAMPLE_RATE: f32 = 16_000.0;
/// Seuil Silero calibré FR bureau
const SILERO_THRESHOLD: f32 = 0.3;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Chemin vers le modèle ONNX (relatif au répertoire du crate).
fn model_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("models")
        .join("silero_vad_v4.onnx")
}

/// Génère une trame sinus à 440 Hz (simule un signal vocal).
fn sine_frame(frame_index: usize) -> Vec<f32> {
    (0..FRAME_SAMPLES)
        .map(|i| {
            let t = (frame_index * FRAME_SAMPLES + i) as f32 / SAMPLE_RATE;
            (2.0 * std::f32::consts::PI * SINE_FREQ_HZ * t).sin() * 0.5
        })
        .collect()
}

/// Génère une trame de silence (tous zéros).
fn silence_frame() -> Vec<f32> {
    vec![0.0f32; FRAME_SAMPLES]
}

/// Calcul du percentile sur un slice trié.
fn percentile(sorted: &[Duration], p: f64) -> Duration {
    assert!(!sorted.is_empty(), "Cannot compute percentile on empty slice");
    let idx = ((sorted.len() as f64 * p / 100.0).ceil() as usize).saturating_sub(1);
    sorted[idx.min(sorted.len() - 1)]
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 1 : benchmark latence VAD (1 000 trames)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn vad_latency_benchmark() {
    let path = model_path();
    if !path.exists() {
        eprintln!(
            "[vad_latency_benchmark] Modèle absent : {} — test ignoré",
            path.display()
        );
        return;
    }

    // Initialisation SileroVad
    let silero = match SileroVad::new(&path, SILERO_THRESHOLD) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[vad_latency_benchmark] Impossible d'initialiser SileroVad : {e} — test ignoré");
            return;
        }
    };

    // Wrap dans SmoothedVad (prefill=15, hangover=15, onset=2 — valeurs Handy)
    let mut vad = SmoothedVad::new(Box::new(silero), 15, 15, 2);

    let n_frames = 1_000usize;
    let mut durations: Vec<Duration> = Vec::with_capacity(n_frames);

    for i in 0..n_frames {
        let frame = sine_frame(i);
        let t = Instant::now();
        let _ = vad
            .push_frame(&frame)
            .expect("push_frame ne doit pas échouer");
        durations.push(t.elapsed());
    }

    // Tri pour percentiles
    durations.sort_unstable();
    let p50 = percentile(&durations, 50.0);
    let p99 = percentile(&durations, 99.0);
    let p90 = percentile(&durations, 90.0);
    let min = durations[0];
    let max = durations[n_frames - 1];

    eprintln!(
        "\n  [VAD Benchmark] SileroVad + SmoothedVad (n={n_frames}, trame 30 ms @ 16 kHz)\n\
         \n  Signal : sinus {SINE_FREQ_HZ} Hz (simulation vocale)\n\
         \n  ┌──────────┬──────────┐\
         \n  │ Stat     │ Latence  │\
         \n  ├──────────┼──────────┤\
         \n  │ min      │ {:>8?} │\
         \n  │ p50      │ {:>8?} │\
         \n  │ p90      │ {:>8?} │\
         \n  │ p99      │ {:>8?} │\
         \n  │ max      │ {:>8?} │\
         \n  └──────────┴──────────┘",
        min, p50, p90, p99, max
    );

    assert!(
        p50 < Duration::from_millis(2),
        "p50 VAD trop élevé : {:?} (seuil 2 ms)",
        p50
    );
    assert!(
        p99 < Duration::from_millis(10),
        "p99 VAD trop élevé : {:?} (seuil 10 ms)",
        p99
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 2 : smoke test — 100 trames de silence → aucune parole détectée
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_silero_vad_smoke() {
    let path = model_path();
    if !path.exists() {
        eprintln!(
            "[test_silero_vad_smoke] Modèle absent : {} — test ignoré",
            path.display()
        );
        return;
    }

    let silero = match SileroVad::new(&path, SILERO_THRESHOLD) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[test_silero_vad_smoke] Impossible d'initialiser SileroVad : {e} — test ignoré");
            return;
        }
    };

    let mut vad = SmoothedVad::new(Box::new(silero), 15, 15, 2);

    let silence = silence_frame();
    let mut speech_detected = false;

    for _ in 0..100 {
        let frame = vad
            .push_frame(&silence)
            .expect("push_frame ne doit pas échouer sur silence");
        if frame.is_speech() {
            speech_detected = true;
            break;
        }
    }

    assert!(
        !speech_detected,
        "VAD a détecté de la parole sur 100 trames de silence — faux positif"
    );

    eprintln!("[test_silero_vad_smoke] OK — aucune parole détectée sur 100 trames de silence");
}
