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
 * Lance une transcription avec des params optimisés.
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

    p->language         = language;
    p->translate        = translate;
    p->single_segment   = false;
    p->print_special    = false;
    p->print_progress   = false;
    p->print_realtime   = false;
    p->print_timestamps = false;
    p->no_context       = false;

    int ret = whisper_full(ctx, *p, samples, n_samples);
    whisper_free_params(p);
    return ret;
}
