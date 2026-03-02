//! Single-key trigger system (Story 7.1)
//!
//! Écoute une seule touche modificatrice (Option ou Commande) et implémente :
//! - **PTT (Push-to-Talk)** : maintien > 200ms → enregistrement → relâchement → stop
//! - **Mains libres (Double-tap)** : double-tap rapide → enregistrement continu → tap → stop
//!
//! ## Architecture threads
//!
//! ```text
//! Thread rdev (callback ultra-léger) ──mpsc──► Thread state machine
//!                                                       │
//!                                                       ▼
//!                                              TranscriptionCoordinator
//! ```
//!
//! Le callback rdev ne fait que `tx.send(event)` — zéro mutex, zéro I/O.
//! La touche courante est lue via un `AtomicU8` (load lock-free, sans blocage CGEventTap).
//! Toute la logique de la machine à états tourne dans le thread séparé.

use log::{debug, error, info, warn};
use rdev::{listen, Event, EventType, Key};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

use crate::transcription_coordinator::TranscriptionCoordinator;

/// Délai d'intention PTT : si Option est maintenu plus longtemps, on démarre l'enregistrement
const INTENT_DELAY_MS: u64 = 200;

/// Fenêtre double-tap : temps maximum entre deux taps pour déclencher le mode mains libres
const DOUBLE_TAP_WINDOW_MS: u64 = 300;

/// Valeur atomique pour la touche Option (⌥)
const TRIGGER_KEY_OPTION: u8 = 0;
/// Valeur atomique pour la touche Commande (⌘)
const TRIGGER_KEY_COMMAND: u8 = 1;

// ============================================================================
// Types internes
// ============================================================================

#[derive(Debug, Clone)]
enum RawKeyEvent {
    TriggerKeyDown,
    TriggerKeyUp,
    OtherKeyDown,
}

enum SmState {
    Idle,
    Pending { t0: Instant },
    WaitingDoubleTap { deadline: Instant },
    RecordingPtt,
    HandsFree,
}

// ============================================================================
// État partagé (pour changer la touche à chaud)
// ============================================================================

/// État géré par Tauri permettant de mettre à jour la touche de déclenchement à la volée.
///
/// Utilise un `AtomicU8` pour les mises à jour lock-free depuis le callback rdev
/// (CGEventTap sur macOS est sensible à tout blocage dans le callback).
pub struct SingleKeyState {
    pub trigger_key: Arc<AtomicU8>,
}

impl SingleKeyState {
    /// Met à jour la touche de déclenchement. Prise en compte immédiatement par le thread rdev.
    pub fn update_trigger_key(&self, key: String) {
        let val = match key.as_str() {
            "command" => TRIGGER_KEY_COMMAND,
            _ => TRIGGER_KEY_OPTION,
        };
        self.trigger_key.store(val, Ordering::Relaxed);
    }
}

// ============================================================================
// Mapping touche → rdev Key (lock-free : prend un u8 atomic)
// ============================================================================

fn is_trigger_key(key: &Key, trigger: u8) -> bool {
    match trigger {
        TRIGGER_KEY_COMMAND => matches!(key, Key::MetaLeft | Key::MetaRight),
        _ => matches!(key, Key::Alt | Key::AltGr),
    }
}

// ============================================================================
// Communication avec le coordinateur
// ============================================================================

/// Démarre l'enregistrement en mode PTT (push_to_talk=true, is_pressed=true).
fn send_start_ptt(app: &AppHandle) {
    if let Some(coordinator) = app.try_state::<TranscriptionCoordinator>() {
        coordinator.send_input("transcribe", "single_key", true, true);
    } else {
        warn!("single_key: TranscriptionCoordinator non disponible (PTT start)");
    }
}

/// Arrête l'enregistrement PTT (push_to_talk=true, is_pressed=false).
fn send_stop_ptt(app: &AppHandle) {
    if let Some(coordinator) = app.try_state::<TranscriptionCoordinator>() {
        coordinator.send_input("transcribe", "single_key", false, true);
    } else {
        warn!("single_key: TranscriptionCoordinator non disponible (PTT stop)");
    }
}

/// Bascule l'état mains libres (push_to_talk=false, is_pressed=true).
/// - Premier appel → démarre l'enregistrement
/// - Second appel → arrête l'enregistrement
fn send_toggle_handsfree(app: &AppHandle) {
    if let Some(coordinator) = app.try_state::<TranscriptionCoordinator>() {
        coordinator.send_input("transcribe", "single_key", true, false);
    } else {
        warn!("single_key: TranscriptionCoordinator non disponible (HandsFree toggle)");
    }
}

// ============================================================================
// Démarrage du listener
// ============================================================================

/// Lance le listener single-key. Appelé depuis `shortcut::init_shortcuts`.
///
/// Spawne deux threads :
/// 1. Thread rdev — callback ultra-léger : lit `AtomicU8` (lock-free), envoie via mpsc
/// 2. Thread state machine — gère les timers, transitions d'états et appels au coordinator
pub fn start_single_key_listener(app: AppHandle, trigger_key: String) {
    let initial_val = match trigger_key.as_str() {
        "command" => TRIGGER_KEY_COMMAND,
        _ => TRIGGER_KEY_OPTION,
    };

    let trigger_atomic = Arc::new(AtomicU8::new(initial_val));
    let trigger_for_rdev = Arc::clone(&trigger_atomic);

    // Stocker dans le state Tauri pour les mises à jour à chaud
    app.manage(SingleKeyState {
        trigger_key: Arc::clone(&trigger_atomic),
    });

    // Canal rdev → state machine
    let (tx, rx) = std::sync::mpsc::channel::<RawKeyEvent>();

    // ------------------------------------------------------------------
    // Thread 1 : rdev listener
    // Le callback est intentionnellement minimal : une lecture AtomicU8 + tx.send().
    // Aucun mutex, aucun I/O — macOS CGEventTap ne se bloque pas.
    // ------------------------------------------------------------------
    thread::spawn(move || {
        info!("single_key: thread rdev démarré");
        if let Err(e) = listen(move |event: Event| {
            let trigger = trigger_for_rdev.load(Ordering::Relaxed);
            match event.event_type {
                EventType::KeyPress(key) => {
                    if is_trigger_key(&key, trigger) {
                        let _ = tx.send(RawKeyEvent::TriggerKeyDown);
                    } else {
                        let _ = tx.send(RawKeyEvent::OtherKeyDown);
                    }
                }
                EventType::KeyRelease(key) => {
                    if is_trigger_key(&key, trigger) {
                        let _ = tx.send(RawKeyEvent::TriggerKeyUp);
                    }
                }
                _ => {}
            }
        }) {
            error!("single_key: rdev::listen erreur : {:?}", e);
        }
    });

    // ------------------------------------------------------------------
    // Thread 2 : state machine
    // ------------------------------------------------------------------
    let app_clone = app.clone();
    thread::spawn(move || {
        info!("single_key: thread state machine démarré");
        let mut sm = SmState::Idle;

        loop {
            // Calcul du timeout selon l'état courant
            let timeout = match &sm {
                SmState::Idle | SmState::RecordingPtt | SmState::HandsFree => {
                    Duration::from_millis(50)
                }
                SmState::Pending { t0 } => {
                    let elapsed = t0.elapsed();
                    if elapsed >= Duration::from_millis(INTENT_DELAY_MS) {
                        Duration::from_millis(0)
                    } else {
                        Duration::from_millis(INTENT_DELAY_MS) - elapsed
                    }
                }
                SmState::WaitingDoubleTap { deadline } => {
                    let now = Instant::now();
                    if *deadline <= now {
                        Duration::from_millis(0)
                    } else {
                        *deadline - now
                    }
                }
            };

            // Tenter de recevoir un event.
            // Priorité aux events déjà en attente (try_recv), puis attente bornée.
            let event = rx.try_recv().ok().or_else(|| {
                if timeout.is_zero() {
                    None // Timer expiré
                } else {
                    rx.recv_timeout(timeout).ok()
                }
            });

            // Transitions d'états
            sm = match sm {
                // ── Idle ────────────────────────────────────────────────────
                SmState::Idle => match event {
                    Some(RawKeyEvent::TriggerKeyDown) => {
                        debug!("single_key: Idle → Pending");
                        SmState::Pending { t0: Instant::now() }
                    }
                    _ => SmState::Idle,
                },

                // ── Pending ─────────────────────────────────────────────────
                SmState::Pending { t0 } => match event {
                    Some(RawKeyEvent::OtherKeyDown) => {
                        debug!("single_key: Pending → Idle (autre touche, annulation silencieuse)");
                        SmState::Idle
                    }
                    Some(RawKeyEvent::TriggerKeyUp) => {
                        if t0.elapsed() < Duration::from_millis(INTENT_DELAY_MS) {
                            debug!("single_key: Pending → WaitingDoubleTap");
                            SmState::WaitingDoubleTap {
                                deadline: Instant::now()
                                    + Duration::from_millis(DOUBLE_TAP_WINDOW_MS),
                            }
                        } else {
                            debug!("single_key: Pending → Idle (relâchement tardif)");
                            SmState::Idle
                        }
                    }
                    Some(RawKeyEvent::TriggerKeyDown) => {
                        // Répétition OS, on ignore mais on garde le t0 original
                        SmState::Pending { t0 }
                    }
                    None => {
                        // Timer 200ms expiré → intention PTT confirmée
                        debug!("single_key: Pending → RecordingPtt (intention confirmée)");
                        send_start_ptt(&app_clone);
                        SmState::RecordingPtt
                    }
                },

                // ── WaitingDoubleTap ─────────────────────────────────────────
                SmState::WaitingDoubleTap { deadline } => match event {
                    Some(RawKeyEvent::TriggerKeyDown) => {
                        debug!("single_key: WaitingDoubleTap → HandsFree");
                        send_toggle_handsfree(&app_clone);
                        SmState::HandsFree
                    }
                    None => {
                        debug!(
                            "single_key: WaitingDoubleTap → Idle (tap simple ignoré, timer expiré)"
                        );
                        SmState::Idle
                    }
                    _ => SmState::WaitingDoubleTap { deadline },
                },

                // ── RecordingPtt ─────────────────────────────────────────────
                SmState::RecordingPtt => match event {
                    Some(RawKeyEvent::TriggerKeyUp) => {
                        debug!("single_key: RecordingPtt → Idle (relâchement → stop)");
                        send_stop_ptt(&app_clone);
                        SmState::Idle
                    }
                    _ => SmState::RecordingPtt,
                },

                // ── HandsFree ────────────────────────────────────────────────
                SmState::HandsFree => match event {
                    Some(RawKeyEvent::TriggerKeyDown) => {
                        debug!("single_key: HandsFree → Idle (tap → stop)");
                        send_toggle_handsfree(&app_clone);
                        SmState::Idle
                    }
                    _ => SmState::HandsFree,
                },
            };
        }
    });

    info!(
        "single_key: listener démarré avec trigger_key='{}'",
        trigger_key
    );
}

// ============================================================================
// Tests unitaires
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_matches_alt_keys() {
        assert!(is_trigger_key(&Key::Alt, TRIGGER_KEY_OPTION));
        assert!(is_trigger_key(&Key::AltGr, TRIGGER_KEY_OPTION));
        assert!(!is_trigger_key(&Key::MetaLeft, TRIGGER_KEY_OPTION));
        assert!(!is_trigger_key(&Key::MetaRight, TRIGGER_KEY_OPTION));
    }

    #[test]
    fn command_matches_meta_keys() {
        assert!(is_trigger_key(&Key::MetaLeft, TRIGGER_KEY_COMMAND));
        assert!(is_trigger_key(&Key::MetaRight, TRIGGER_KEY_COMMAND));
        assert!(!is_trigger_key(&Key::Alt, TRIGGER_KEY_COMMAND));
        assert!(!is_trigger_key(&Key::AltGr, TRIGGER_KEY_COMMAND));
    }

    #[test]
    fn unknown_trigger_defaults_to_option() {
        // Toute valeur autre que TRIGGER_KEY_COMMAND doit matcher Option
        assert!(is_trigger_key(&Key::Alt, 99));
        assert!(!is_trigger_key(&Key::MetaLeft, 99));
    }

    #[test]
    fn intent_delay_constant_is_200ms() {
        assert_eq!(INTENT_DELAY_MS, 200);
    }

    #[test]
    fn double_tap_window_constant_is_300ms() {
        assert_eq!(DOUBLE_TAP_WINDOW_MS, 300);
    }
}
