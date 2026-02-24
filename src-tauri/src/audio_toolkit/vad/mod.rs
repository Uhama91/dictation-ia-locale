//! # VAD — Voice Activity Detection
//!
//! ## VAD retenu : Silero v4 via `vad-rs`
//!
//! Silero VAD v4 (ONNX) est exécuté localement via le crate `vad-rs`.
//! Inférence CPU-only, latence cible p50 < 2 ms / p99 < 10 ms pour une trame 30 ms à 16 kHz.
//!
//! ## Alternatives évaluées et écartées
//!
//! - **TEN VAD** : évalué mais écarté — pas de binding Rust natif disponible ;
//!   le wrapping de la bibliothèque C aurait nécessité une couche FFI non triviale,
//!   trop complexe pour un MVP. Silero couvre le même besoin avec un crate prêt à l'emploi.
//!
//! ## Paramètres `SmoothedVad` retenus
//!
//! | Paramètre | Valeur | Justification |
//! |-----------|--------|---------------|
//! | `prefill`  | 15 frames | ~450 ms de pré-roll conservé (valeurs Handy) |
//! | `hangover` | 15 frames | ~450 ms de queue après fin de parole (valeurs Handy) |
//! | `onset`    |  2 frames | ~60 ms pour confirmer le début de parole (valeurs Handy) |
//!
//! ## Seuil Silero
//!
//! `threshold = 0.3` — calibré pour le français en environnement bureau (bureau calme à
//! semi-bruyant). Une valeur plus haute (0.5) génère trop de faux négatifs sur les voix
//! féminines douces ; une valeur plus basse (0.2) déclenche trop de faux positifs avec le
//! bruit de clavier.

use anyhow::Result;

pub enum VadFrame<'a> {
    /// Speech – may aggregate several frames (prefill + current + hangover)
    Speech(&'a [f32]),
    /// Non-speech (silence, noise). Down-stream code can ignore it.
    Noise,
}

impl<'a> VadFrame<'a> {
    #[inline]
    pub fn is_speech(&self) -> bool {
        matches!(self, VadFrame::Speech(_))
    }
}

pub trait VoiceActivityDetector: Send + Sync {
    /// Primary streaming API: feed one 30-ms frame, get keep/drop decision.
    fn push_frame<'a>(&'a mut self, frame: &'a [f32]) -> Result<VadFrame<'a>>;

    fn is_voice(&mut self, frame: &[f32]) -> Result<bool> {
        Ok(self.push_frame(frame)?.is_speech())
    }

    fn reset(&mut self) {}
}

mod silero;
mod smoothed;

pub use silero::SileroVad;
pub use smoothed::SmoothedVad;
