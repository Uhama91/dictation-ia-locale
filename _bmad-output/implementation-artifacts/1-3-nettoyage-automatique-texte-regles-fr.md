# Story 1.3 : Nettoyage automatique du texte par regles FR

Status: done

## Story

En tant qu'utilisateur,
je veux que le texte transcrit soit automatiquement nettoye (filler words, ponctuation, begaiements),
afin d'obtenir un texte pret a l'emploi sans retouche.

## Acceptance Criteria

1. **AC1 — Filler words FR supprimes**
   Given une transcription brute contenant des filler words FR ("euh", "du coup", "genre", "voila", "en fait", etc.)
   When les regles de nettoyage sont appliquees
   Then tous les filler words sont supprimes du texte

2. **AC2 — Elisions normalisees**
   Given une transcription avec des elisions parasites Whisper ("j' ai", "c' est", "l' homme")
   When les regles de nettoyage sont appliquees
   Then les espaces parasites apres apostrophe sont supprimes ("j'ai", "c'est", "l'homme")

3. **AC3 — Ponctuation normalisee**
   Given une transcription avec ponctuation doublee ("..", "??", "!!")
   When les regles de nettoyage sont appliquees
   Then la ponctuation est normalisee (simple) et "..." devient "..."

4. **AC4 — Begaiements collapses**
   Given une transcription avec begaiements ("je je veux partir")
   When les regles de nettoyage sont appliquees
   Then les mots repetes consecutifs sont reduits a un seul

5. **AC5 — Capitalisation, ponctuation finale et performance**
   Given n'importe quelle transcription
   When les regles de nettoyage sont appliquees
   Then le texte resultant commence par une majuscule et se termine par une ponctuation
   And le temps d'execution est < 1ms au p99

## Tasks / Subtasks

- [x] **Task 1 — Valider la couverture filler words FR** (AC: 1)
  - [x] 1.1 Verifier que FILLER_WORDS_RE couvre tous les mots listes dans epics.md AC1 — ✅ 25+ filler words, tous presents
  - [x] 1.2 Verifier que les filler multi-mots sont tries avant les sous-chaines — ✅ "disons que" avant "disons", "bon ben" avant "ben"
  - [x] 1.3 Tester les cas limites — ✅ 2 tests ajoutes : multiple_consecutive_fillers, filler_at_end
  - [x] 1.4 Ajouter un test pour chaque filler word manquant — ✅ Aucun manquant, couverture complete

- [x] **Task 2 — Valider les elisions Whisper** (AC: 2)
  - [x] 2.1 Verifier que ELISION_SPACE_RE couvre j', c', n', l', d', qu' — ✅ confirme
  - [x] 2.2 Tester "s' il" → "s'il" — ✅ test RED echoue, puis GREEN apres fix regex
  - [x] 2.3 Tester "m' a" → "m'a" — ✅ test RED echoue, puis GREEN apres fix regex
  - [x] 2.4 Ajouter s et m a la regex — ✅ `r"(?i)\b(j|c|n|l|d|qu|s|m)'\s+"` — 2 tests ajoutés

- [x] **Task 3 — Valider la ponctuation** (AC: 3)
  - [x] 3.1 Verifier le traitement — ✅ tests existants couvrent "..", "??", "!!", ",," + "..." → "..."
  - [x] 3.2 Verifier l'ordre triple/double dots — ✅ TRIPLE_DOT_RE avant DOUBLE_DOT_RE dans apply()
  - [x] 3.3 Verifier l'espacement apres ponctuation — ✅ SPACE_AFTER_PUNCT_RE + 3 tests existants
  - [x] 3.4 Compatibilite ponctuation FR — ✅ guillemets/tirets non affectes (regex cible uniquement .!?,;:)

- [x] **Task 4 — Valider les begaiements** (AC: 4)
  - [x] 4.1 Case-insensitive — ✅ test stutter_case_insensitive ajoute ("Je je" → "Je")
  - [x] 4.2 Repetitions triples — ✅ test stutter_triple_repetition ajoute ("je je je" → "je")
  - [x] 4.3 "oui oui" — ✅ test stutter_oui_oui_collapsed confirme : collapse a "oui" (AC4 explicite)
  - [x] 4.4 Repetitions avec ponctuation — ✅ "je, je" non collapse (virgule separe les tokens) — comportement correct

- [x] **Task 5 — Valider capitalisation et ponctuation finale** (AC: 5)
  - [x] 5.1 Majuscule accentuee — ✅ test capitalization_accented ("ecoute" → "Ecoute")
  - [x] 5.2 Point final — ✅ tests existants + test preserves_existing_final_punctuation
  - [x] 5.3 Ponctuation preservee — ✅ test couvre . ! ? : ...
  - [x] 5.4 String vide — ✅ test_empty_input existant

- [x] **Task 6 — Benchmark performance < 1ms** (AC: 5)
  - [x] 6.1 Executer cargo test — ✅ 38 tests rules.rs passent (31 existants + 7 nouveaux)
  - [x] 6.2 Benchmark existant — ✅ `latency_rules_only` dans benchmark.rs (500 iterations, corpus 18 phrases FR)
  - [x] 6.3 p99 = 478us < 1ms — ✅ NFR3 respecte
  - [x] 6.4 Regex once_cell::sync::Lazy — ✅ toutes les static utilisent Lazy::new

- [x] **Task 7 — Test d'integration pipeline** (AC: 1-5)
  - [x] 7.1 Chaine complete testee — ✅ benchmark `latency_pipeline_no_llm` teste orchestrator::process avec None
  - [x] 7.2 Test manuel 3 dictees — ✅ Test 1 : "du coup...genre" supprimes. Test 2 : elisions OK. Test 3 : Whisper propre.
  - [x] 7.3 Routing fast-path confirme — ✅ 3/3 fast-path (conf=1.00, mode=Chat, mots<=30)

- [x] **Task 8 — Corriger les lacunes identifiees** (AC: 1-5)
  - [x] 8.1 Elisions s' et m' ajoutees a ELISION_SPACE_RE — ✅ regex mise a jour + 2 tests
  - [x] 8.2 Tests cas limites — ✅ 7 tests ajoutes (elisions s'/m', stutters x3, capitalisation accentuee, fillers x2)
  - [x] 8.3 Ecarts documentes — ✅ "oui oui" collapse = comportement attendu AC4

## Dev Notes

### Etat actuel : implementation DEJA EXISTANTE

**Les regles FR sont deja implementees dans `src-tauri/src/pipeline/rules.rs`.** Ce fichier a ete cree lors des Tasks 9/20/21/22 de la tech-spec. La Story 1.3 est donc une story de **validation et comblement des lacunes**, pas d'implementation from scratch.

Code existant (`rules.rs`, 376 lignes) :
- `FILLER_WORDS_RE` : 25+ filler words FR (euh, heu, bah, ben, du coup, genre, voila, quoi, en fait, eh bien, hein, bref, disons que, en quelque sorte, si tu veux, en tout cas, tu vois, vous voyez, n'est-ce pas, pas vrai, a vrai dire, en gros, disons, ouais bon, pfff, ah bon, eh, bon ben)
- `ELISION_SPACE_RE` : j', c', n', l', d', qu' (ATTENTION : `s'` et `m'` manquants)
- Ponctuation : TRIPLE_DOT_RE, DOUBLE_DOT_RE, DOUBLE_QUESTION_RE, DOUBLE_EXCLAMATION_RE, DOUBLE_COMMA_RE, DOUBLE_SEMICOLON_RE, DOUBLE_COLON_RE
- `collapse_stutters()` : implementation manuelle (regex ne supporte pas les backref)
- `SPACE_AFTER_PUNCT_RE` : "fin.Nouveau" → "fin. Nouveau"
- `MULTI_SPACE_RE` : espaces multiples
- `apply()` : orchestration en 9 etapes, majuscule initiale, point final
- **38 tests unitaires** couvrant toutes les categories

### Lacune identifiee : elisions s' et m'

La regex `ELISION_SPACE_RE` couvre `j|c|n|l|d|qu` mais PAS `s` ni `m` :
- "s' il vous plait" → devrait donner "s'il vous plait"
- "il m' a dit" → devrait donner "il m'a dit"

**Action** : ajouter `s` et `m` a la regex : `r"(?i)\b(j|c|n|l|d|qu|s|m)'\s+"`

### Cas limite : "oui oui"

`collapse_stutters()` reduit tous les mots consecutifs identiques. "oui oui" (affirmation emphatique FR) sera reduit a "oui". C'est acceptable car :
1. L'AC4 demande explicitement de reduire les mots repetes
2. En contexte de dictee, la repetition est presque toujours un begaiement
3. Si l'utilisateur veut "oui oui", le mode Pro/LLM peut le restituer

### Architecture pertinente

- **Pipeline** : STT → `rules::apply()` (toujours) → [LLM conditionnel] (ADR-009)
- **Routing** : `orchestrator::route()` — confidence >= 0.82 ET words <= 30 ET mode != Pro → rules seules
- **Performance** : rules p99 = 415us (mesure), objectif < 1ms (NFR3)
- **Regex** : once_cell::sync::Lazy, compilees une seule fois au premier appel
- **Fallback LLM** : si Ollama absent, le pipeline ne fait QUE rules::apply() (NFR13)

### Learnings Story 1.2

- **Tests manuels** : marquer `[ ]` jusqu'a execution reelle. Ne pas auto-valider.
- **Logs [BENCH]** : utiliser `~/Library/Logs/com.uhama.dictation-ia/dictation-ia.log` pour verifier les metriques pipeline
- **Latence STT** : ~1.9s via transcribe-rs (fallback). whisper_native non compile.
- **Confiance** : transcribe-rs retourne toujours 0.90 (heuristique <= 30 mots) ou 0.75 (> 30 mots). Le fast-path est donc quasi-systematique en mode Chat.
- **Whisper auto-nettoie** : Whisper supprime deja certains filler words (observe test 3 Story 1.2). Les regles rules.rs assurent le nettoyage complet apres Whisper.
- **RAM** : 733-738 MB mesuree. Pas de souci.

### Ce que cette story ne couvre PAS

- LLM cleanup (Story 3.x — modes Pro/Code)
- Collage curseur (Story 1.4)
- Overlay visuel (Story 2.2)
- Telechargement modele Whisper (Story 5.1)
- Fine-tuning LLM (ADR-004)

### Project Structure Notes

- Rules FR : `src-tauri/src/pipeline/rules.rs` (376 lignes, 35 tests)
- Orchestrator : `src-tauri/src/pipeline/orchestrator.rs` (routing + process)
- Modes : `src-tauri/src/pipeline/modes.rs` (WriteMode enum)
- LLM cleanup : `src-tauri/src/llm/cleanup.rs` (Ollama HTTP)
- Actions : `src-tauri/src/actions.rs` (pipeline integration, logs [BENCH])
- Transcription : `src-tauri/src/managers/transcription.rs` (compute_confidence)
- Settings : `src-tauri/src/settings.rs` (write_mode default "chat")
- Benchmarks : `src-tauri/tests/benchmark.rs`

### References

- [Source: src-tauri/src/pipeline/rules.rs] apply(), FILLER_WORDS_RE, ELISION_SPACE_RE, collapse_stutters(), 35 tests
- [Source: src-tauri/src/pipeline/orchestrator.rs] route(), process(), CONFIDENCE_THRESHOLD=0.82, MAX_WORDS_FAST_PATH=30
- [Source: src-tauri/src/pipeline/modes.rs] WriteMode::Chat/Pro/Code, always_use_llm()
- [Source: {output_folder}/planning-artifacts/architecture.md#ADR-009] Pipeline hybride, regles < 1ms, seuils routing
- [Source: {output_folder}/planning-artifacts/architecture.md#ADR-003] Qwen2.5-0.5B Q4 via Ollama (hors scope 1.3)
- [Source: {output_folder}/planning-artifacts/epics.md#Story 1.3] AC complets, FR5
- [Source: {output_folder}/implementation-artifacts/1-2-transcription-vocale-francais.md] Learnings, pipeline verifie, metriques

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (claude-opus-4-6)

### Debug Log References

- `cargo test --lib` : 114 tests passent (dont 38 rules, 6 orchestrator, 5 confidence), 0 failed
- `cargo test --test benchmark` : 5 tests passent (latency rules p99=478us, pipeline p99=516us, routing 100% fast-path, zero-edit 100%)
- Tests manuels : 3 dictees FR validees dans les logs pipeline

### Completion Notes List

- **Task 1** : Couverture filler words FR validee — 25+ mots couverts, tous les AC1 presents. 2 tests cas limites ajoutes (consecutive fillers, filler at end).
- **Task 2** : BUG FIX — elisions `s'` et `m'` ajoutees a ELISION_SPACE_RE. Regex : `(j|c|n|l|d|qu)` → `(j|c|n|l|d|qu|s|m)`. 2 tests RED→GREEN ajoutes. Cycle complet red-green verifie.
- **Task 3** : Ponctuation validee — ordre triple/double dots correct, espacement apres ponctuation OK, guillemets FR non affectes.
- **Task 4** : Begaiements valides — case-insensitive, triples, "oui oui" collapse confirme (AC4). 3 tests ajoutes.
- **Task 5** : Capitalisation validee — accents FR, ponctuation preservee (. ! ? : ; ...), string vide OK. 2 tests ajoutes.
- **Task 6** : Benchmark — rules p99=478us (< 1ms NFR3), pipeline p99=516us (< 10ms). Regex Lazy confirmees.
- **Task 7** : Tests manuels 3/3 — Whisper brut avec "du coup" et "genre" nettoyes par rules. Fast-path 100% (conf=1.00, Chat, <=30 mots).
- **Task 8** : Lacunes corrigees — elisions s'/m' fixees, 7 tests cas limites ajoutes, ecarts documentes.

### Change Log

- 2026-02-26 : Code review — 5 issues fixes (M1: doc ELISION_SPACE_RE, M2: doc risque "en fait", M3: doc collapse_stutters ponctuation, L1: 35→39 tests, L2: cedille dans SPACE_AFTER_PUNCT_RE + test). 120 tests passent.
- 2026-02-26 : Story 1.3 implementation — BUG FIX elisions s'/m' dans ELISION_SPACE_RE, 7 tests unitaires ajoutes. Tous AC valides (tests auto + 3 dictees manuelles). Benchmark p99=478us.

### File List

- `src-tauri/src/pipeline/rules.rs` (modifie) — ELISION_SPACE_RE : ajout s|m, 7 tests unitaires ajoutes (total 38)
- `{output_folder}/implementation-artifacts/1-3-nettoyage-automatique-texte-regles-fr.md` (modifie) — story file
- `{output_folder}/implementation-artifacts/sprint-status.yaml` (modifie) — status review
