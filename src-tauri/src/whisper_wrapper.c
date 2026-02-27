/**
 * whisper_wrapper.c — Wrapper C pour whisper.cpp
 *
 * Encapsule la configuration de whisper_full_params pour éviter
 * de devoir reproduire l'exact layout de la struct en Rust.
 *
 * Compilé par build.rs avec la crate `cc`.
 */

#include <stdbool.h>
#include "whisper.h"

/**
 * Lance une transcription avec des params optimisés pour la latence.
 *
 * @param ctx      Contexte whisper (de whisper_init_from_file)
 * @param language Code langue ISO (ex: "fr", "en") — NULL = auto-détection
 * @param translate true = traduire vers l'anglais
 * @param samples  Samples audio PCM f32 mono 16kHz
 * @param n_samples Nombre de samples
 * @return Code retour whisper_full() — 0 = succès
 */
int whisper_run(
    struct whisper_context* ctx,
    const char*             language,
    bool                    translate,
    const float*            samples,
    int                     n_samples
) {
    struct whisper_full_params* p =
        whisper_full_default_params_by_ref(WHISPER_SAMPLING_GREEDY);

    // --- Langue et mode ---
    p->language         = language;
    p->translate        = translate;

    // --- Optimisations latence ---
    p->no_context       = true;   // Pas d'historique entre segments (dictée = phrases isolées)
    p->single_segment   = true;   // Un seul segment — évite le découpage multi-segment
    p->n_threads        = 4;      // M1 = 4 perf cores, saturer les cores efficaces
    p->flash_attn       = true;   // Flash attention — réduit la mémoire et accélère l'inférence

    // --- Greedy optimisé ---
    p->greedy.best_of   = 1;      // Pas de beam search, un seul candidat (plus rapide)

    // --- Réduction du travail inutile ---
    p->suppress_blank   = true;   // Supprime les tokens vides
    p->suppress_nst     = true;   // Supprime les tokens non-speech
    p->no_timestamps    = true;   // Pas de timestamps (on n'en a pas besoin)

    // --- Désactiver toute sortie console ---
    p->print_special    = false;
    p->print_progress   = false;
    p->print_realtime   = false;
    p->print_timestamps = false;

    int ret = whisper_full(ctx, *p, samples, n_samples);
    whisper_free_params(p);
    return ret;
}
