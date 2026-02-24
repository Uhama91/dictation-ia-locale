---
title: 'Dictation IA Locale — Outil de dictée vocale francophone local-first'
slug: 'dictation-ia-locale-mvp'
created: '2026-02-24'
status: 'ready-for-dev'
stepsCompleted: [1, 2, 3, 4]
tech_stack:
  - 'Rust'
  - 'Tauri 2.x'
  - 'whisper.cpp (FFI Rust) avec WHISPER_COREML=1 + WHISPER_METAL=1'
  - 'Whisper large-v3-turbo Q5_0'
  - 'Qwen2.5-0.5B Q4 via llama-cpp-rs'
  - 'TEN VAD (ten-vad)'
  - 'Swift plugin (FFI depuis Rust) pour ANE + menu bar + Accessibility'
  - 'SQLite (historique)'
  - 'React/TypeScript (UI settings)'
files_to_modify: []
files_to_create:
  - 'src-tauri/src/managers/audio.rs'
  - 'src-tauri/src/managers/transcription.rs'
  - 'src-tauri/src/managers/model.rs'
  - 'src-tauri/src/managers/history.rs'
  - 'src-tauri/src/audio_toolkit/'
  - 'src-tauri/src/pipeline/orchestrator.rs'
  - 'src-tauri/src/pipeline/rules.rs'
  - 'src-tauri/src/pipeline/modes.rs'
  - 'src-tauri/src/llm/cleanup.rs'
  - 'src-tauri/src/whisper_ffi.rs'
  - 'src-tauri/src/input.rs'
  - 'src-tauri/src/shortcut/'
  - 'src-tauri/swift-plugin/WhisperANE.swift'
  - 'src-tauri/swift-plugin/MenuBar.swift'
  - 'src-tauri/swift-plugin/AccessibilityPaste.swift'
  - 'src/components/Settings.tsx'
  - 'tests/benchmark.rs'
code_patterns:
  - 'Handy fork — architecture Rust + Tauri 2.x conservée'
  - 'Pipeline hybride : règles < 1ms → LLM conditionnel (si score confiance bas)'
  - 'whisper.cpp via FFI Rust (pas transcribe-rs) pour accès CoreML encoder'
  - 'Swift plugin via FFI depuis Rust (Accessibility + ANE + NSStatusBar)'
  - 'Tauri IPC pour communication Rust ↔ React settings'
  - 'Async Tokio channels pour orchestration pipeline'
  - 'Chargement à la demande des modèles + déchargement après timeout'
test_patterns:
  - 'Unit tests Rust par module (cargo test)'
  - 'Benchmark latence p50/p99 + zero-edit rate sur 100 phrases FR (tests/benchmark.rs)'
  - 'Tests intégration pipeline complet (audio fixture → texte final)'
---

# Tech-Spec: Dictation IA Locale — Outil de dictée vocale francophone local-first

**Created:** 2026-02-24

## Overview

### Problem Statement

Les outils de dictée vocale performants (Wispr Flow) sont cloud-only et anglophones-first. Wispr Flow atteint une latence < 700ms (p99) et un zero-edit rate de 85%, mais nécessite une connexion internet permanente et envoie l'audio sur des serveurs distants.

Les alternatives open source (Handy) fonctionnent en local mais :
- N'ont pas de couche LLM intégrée pour le nettoyage du texte transcrit
- Ne sont pas optimisées pour le français (modèles par défaut anglophones)
- Le post-traitement LLM est optionnel et nécessite une API cloud externe

Il n'existe pas d'outil de dictée **francophone-first**, **100% offline**, **rapide**, avec **nettoyage IA intégré**.

### Solution

Fork et adaptation de **Handy** (Rust/Tauri 2.x) comme base desktop, en y intégrant un pipeline complet et optimisé :

```
Raccourci clavier → Capture audio (micro) → VAD (Silero) → STT (Whisper quantisé, optimisé FR) → LLM nettoyage embarqué (~1-1.5B fine-tuné FR) → Collage au curseur
```

**Objectifs de performance :**
- Se rapprocher de la fluidité de Wispr Flow (< 700ms) en 100% local
- Tourner sur des machines avec 6-8 Go de RAM
- Zero-edit rate > 70% en français

### Scope

**In Scope (MVP Desktop) :**
- Desktop macOS uniquement (menu bar app via Tauri)
- Pipeline STT local (Whisper GGML quantisé, optimisé français)
- LLM de nettoyage embarqué (~1-1.5B params, fine-tuné pour le nettoyage FR)
- VAD (Silero v4) pour filtrer le silence
- Raccourci clavier global (push-to-talk / toggle)
- Collage automatique au curseur (presse-papier + Cmd+V)
- 3 modes d'écriture : Chat / Pro / Code
- 100% offline, zéro dépendance cloud
- Cible hardware : machines avec 6-8 Go RAM minimum

**Out of Scope (post-MVP) :**
- Application Android (prévue mais séparée, post-MVP)
- Cloud/API opt-in (fallback vers des modèles cloud plus puissants)
- Dictionnaire personnel custom (ajout de mots/jargon)
- Commandes vocales
- Streaming temps réel (batch comme Handy pour le MVP)
- Context-aware par application (adaptation du ton selon l'app active, feature Wispr Flow avancée)
- Diarisation (identification des locuteurs)

## Context for Development

### Projet de référence : Handy (Open Source)

**Repo :** https://github.com/cjpais/Handy
**Stack :** Rust + Tauri 2.x (backend Rust, frontend React/TypeScript)
**Licence :** MIT — forkable, le projet est explicitement conçu pour être "le plus forkable possible"
**Stars :** ~15 800

#### Architecture Handy

```
src-tauri/src/
├── managers/
│   ├── audio.rs          — Gestion enregistrement micro et périphériques
│   ├── transcription.rs  — Pipeline STT (chargement modèle, inférence)
│   ├── model.rs          — Téléchargement et gestion des modèles
│   └── history.rs        — Historique des transcriptions (SQLite)
├── audio_toolkit/        — Traitement audio bas niveau (capture, resampling, VAD)
├── transcription_coordinator.rs — Sérialisation des événements clavier/pipeline
├── actions.rs            — Logique start/stop transcription + post-traitement LLM
├── llm_client.rs         — Client HTTP pour API LLM (post-traitement optionnel)
├── input.rs              — Simulation Ctrl+V / Cmd+V via Enigo
└── shortcut/             — Raccourcis clavier globaux

src/                      — Frontend React/TypeScript (Settings UI)
```

#### Dépendances Rust clés de Handy

| Crate | Rôle |
|-------|------|
| `transcribe-rs` | Moteur STT unifié (Whisper, Parakeet, Moonshine, SenseVoice) |
| `cpal` | Capture audio cross-platform |
| `vad-rs` | Voice Activity Detection (Silero VAD v4) |
| `rubato` | Resampling audio |
| `rdev` | Raccourcis clavier globaux |
| `enigo` | Simulation d'input clavier (collage) |
| `rodio` | Feedback audio (sons de début/fin) |

#### Pipeline Handy actuel (ce qu'on garde)

1. **Raccourci clavier** → `rdev` intercepte, debounce 30ms, modes toggle/push-to-talk
2. **Pre-load modèle** → dès le raccourci pressé, le modèle STT se charge en arrière-plan pendant que l'utilisateur parle
3. **Capture audio** → `cpal` capture en f32 mono, resampling via `rubato` vers 16kHz
4. **VAD SmoothedVad** → Silero VAD v4 + lissage temporal (prefill 15 frames, hangover 15 frames, onset 2 frames)
5. **Transcription batch** → enregistrement complet puis inférence via `transcribe-rs`
6. **Post-traitement texte local** → correction fuzzy (Levenshtein + Soundex + n-grams), filtrage filler words, collapse bégaiements
7. **Collage** → presse-papier système + simulation Cmd+V via Enigo

#### Ce qu'on ajoute (notre valeur ajoutée)

- **LLM de nettoyage embarqué** → intégration de `llama.cpp` (via binding Rust) pour exécuter un petit modèle (~1-1.5B) de nettoyage/reformulation
- **Optimisation francophone** → modèle Whisper configuré/fine-tuné pour le français, prompts LLM en français, dictionnaire de filler words FR
- **3 modes d'écriture natifs** → Chat/Pro/Code avec prompts dédiés, pas de dépendance API externe
- **Budget RAM strict** → choix de modèles quantisés qui tiennent dans 6-8 Go total

### Benchmark de référence : Wispr Flow

| Métrique | Wispr Flow (cloud) | Notre objectif (local) |
|----------|-------------------|----------------------|
| Latence totale | < 700ms (p99) | < 2s (objectif), < 1.5s (stretch) |
| Budget ASR | < 200ms | < 500ms (Whisper small quantisé) |
| Budget LLM | < 200ms (TensorRT-LLM) | < 800ms (llama.cpp, modèle 1-1.5B) |
| Zero-edit rate | 85% | > 70% |
| RAM utilisée | Cloud (illimitée) | < 4 Go (modèles inclus) |
| Vitesse dictée | 220 wpm | 150+ wpm |
| Offline | Non | Oui (100%) |

### Cible technique : Wispr Flow décomposé

Wispr Flow utilise :
- **ASR conditionné** par le contexte (locuteur, historique, app active) — < 200ms
- **Llama fine-tuné** sur Baseten avec **TensorRT-LLM** — < 200ms pour 100+ tokens
- **Pipeline multi-étapes orchestré** via framework Chains (ASR → LLM → formatage)
- **Zero-edit rate 85%** comme métrique principale (pas le WER classique)

Notre stratégie pour reproduire cette qualité en local :
1. **Whisper GGML quantisé** (small ou medium Q4/Q5) — bon compromis qualité FR / vitesse / RAM
2. **Petit LLM spécialisé** (~1-1.5B) fine-tuné exclusivement pour le nettoyage de transcriptions FR
3. **Prompts par mode** (Chat/Pro/Code) injectés dans le LLM pour adapter le style
4. **Post-traitement hybride** : corrections regex locales (filler words FR, bégaiements) AVANT le LLM pour réduire sa charge

### Technical Decisions

#### ADR 001 — Backend : Rust (via fork Handy)
- **Décision :** Rust via fork de Handy/Tauri 2.x
- **Raison :** Performance native, empreinte mémoire minimale, écosystème audio/ML Rust mature (transcribe-rs, vad-rs), cross-platform desktop, base de code éprouvée (15k+ stars)
- **Trade-off :** Courbe d'apprentissage Rust, mais le code Handy est bien structuré et documenté

#### ADR 002 — STT : Whisper large-v3-turbo Q5_0 via whisper.cpp (CoreML + Metal)
- **Décision :** `Whisper large-v3-turbo Q5_0` (~800 Mo) via `whisper.cpp` compilé avec `WHISPER_COREML=1 WHISPER_METAL=1`
- **Raison (unanime — 3 sources) :** Décodeur réduit de 32 → 4 couches = 5.4x plus rapide que large-v3. WER français 3-8% vs 12-15% pour small. Ponctuation et majuscules gérées nativement → réduit la charge du LLM.
- **Abandon de transcribe-rs :** whisper.cpp directement via FFI Rust → accès natif aux optimisations CoreML encoder (ANE 3x) + Metal decoder (3-4x). transcribe-rs ajoutait une abstraction sans gain.
- **Params Whisper optimisés :**
  ```rust
  WhisperConfig { language: "fr", beam_size: 1, best_of: 1, temperature: 0.0, no_speech_threshold: 0.6 }
  ```
- **Latence cible :** ~450-600ms (Encoder ANE ~150ms + Décodeur Metal ~300ms)
- **Budget RAM :** ~800 Mo pour le modèle chargé
- **Alternative future :** Faster-Whisper (CTranslate2) si binding Rust disponible — potentiellement < 300ms

#### ADR 003 — LLM nettoyage : Qwen2.5-0.5B Q4 (pas 1.5B)
- **Décision :** `Qwen2.5-0.5B-Instruct Q4` via `llama-cpp-rs`
- **Raison (unanime — 3 sources) :** La tâche est étroite (correction, non génération créative). Un modèle 0.5B fine-tuné atteint > 70% zero-edit rate. RAM ~300 Mo vs ~900 Mo pour 1.5B.
- **Fallback :** Qwen2.5-1.5B Q4 si le 0.5B s'avère insuffisant après benchmark
- **Params LLM optimisés :**
  ```rust
  LlamaParams { n_predict: 128, temperature: 0.0, top_k: 1, repeat_penalty: 1.0 }
  ```
- **Latence cible :** ~200-300ms pour ~50 tokens
- **Budget RAM :** ~300 Mo (vs ~900 Mo pour 1.5B — économie de 600 Mo)

#### ADR 004 — Fine-tuning du LLM pour le nettoyage FR
- **Approche :** Fine-tuning supervisé (SFT) avec LoRA/QLoRA
- **Dataset à construire :**
  - Paires (transcription brute FR, texte nettoyé FR) pour chaque mode (Chat/Pro/Code)
  - Sources : transcriptions Whisper réelles avec corrections manuelles, datasets open source de correction grammaticale FR
  - Volume cible : 5 000-10 000 paires pour commencer
- **Outils :** Hugging Face Transformers + PEFT (LoRA), ou Unsloth pour l'optimisation
- **Validation :** Zero-edit rate sur un set de test de 500 phrases FR

#### ADR 005 — Framework desktop : Tauri 2.x (hérité de Handy)
- **Décision :** Conserver Tauri 2.x
- **Raison :** Léger (~10 Mo vs ~150 Mo Electron), utilise WebView natif, backend Rust natif, déjà intégré dans Handy
- **UI :** Menu bar macOS natif + fenêtre settings React/TypeScript

#### ADR 006 — Plateforme cible : macOS d'abord, Android séparé (post-MVP)
- **MVP :** macOS uniquement
- **Post-MVP Android :** Runtime séparé en Kotlin + whisper.cpp/llama.cpp via NDK
- **Raison :** Tauri ne supporte pas Android nativement. Mieux vaut livrer un MVP desktop solide avant de fragmenter l'effort.

#### ADR 007 — Architecture hybride : Rust + Plugin Swift (nouveau)
- **Décision :** Ajouter une couche Swift plugin via FFI depuis Rust pour les composants macOS critiques
- **Composants Swift :**
  - `WhisperANE.swift` → Whisper via CoreML encoder sur ANE (3-5x vs GGML Metal seul)
  - `MenuBar.swift` → NSStatusBar natif (vs menu bar Tauri artificielle)
  - `AccessibilityPaste.swift` → AXUIElement API pour collage direct au curseur (sans Enigo)
- **Ce qui reste en Rust :** Orchestration pipeline, VAD, LLM cleanup, async channels Tokio
- **Raison :** Meilleure expérience macOS native sans abandonner le fork Handy. ANE accessible uniquement depuis Swift/CoreML.
- **Trade-off :** Complexité FFI Rust↔Swift, mais gains latence et UX significatifs.
- **V2 :** Migration complète vers Swift natif + CoreML (abandon Tauri/Rust, ~30 Mo RAM vs ~200 Mo)

#### ADR 008 — VAD : TEN VAD (upgrade depuis Silero v4)
- **Décision :** Évaluer `ten-vad` (TEN-framework/ten-vad) vs Silero v4, adopter si benchmark valide
- **Raison :** TEN VAD top 3 des VAD 2025 (benchmark Picovoice). Latence switch speech→silence réduite : gain estimé -100 à -200ms sur la détection de fin de parole.
- **Démarche :** Conserver vad-rs (Silero) comme baseline, benchmark TEN VAD, swap si gain confirmé

#### ADR 009 — Pipeline hybride : règles locales + LLM conditionnel
- **Décision :** Ne pas appeler le LLM systématiquement — pipeline en deux étapes
- **Pipeline :**
  ```
  Transcription Whisper → [Règles locales < 1ms] → Score confiance élevé + phrases courtes ? → Coller direct
                                                   → Score bas / phrases longues / mode Pro/Code → Qwen2.5-0.5B
  ```
- **Règles locales :** Filler words FR (euh, heu, bah, ben, du coup, genre, voilà), collapse bégaiements, majuscule début + point final
- **Résultat attendu :** 60-65% des cas sans LLM (< 5ms), LLM pour 35-40% (~200-300ms)
- **Insight Gemini :** Tester Whisper large-v3-turbo SEUL avant d'implémenter le LLM — zero-edit rate potentiellement > 70% sans LLM du tout sur le FR

### Budget mémoire (RAM) — Contrainte 6-8 Go (mis à jour)

| Composant | RAM estimée | Notes |
|-----------|------------|-------|
| OS + apps en arrière-plan | ~3-4 Go | Incompressible |
| Tauri app (UI + runtime WebView) | ~200 Mo | WebView inclus |
| Modèle Whisper large-v3-turbo Q5_0 | ~800 Mo | Upgrade depuis small/medium — meilleure qualité FR |
| Modèle LLM Qwen2.5-0.5B Q4 | ~300 Mo | Downgrade depuis 1.5B — économie 600 Mo |
| Buffer audio + VAD | ~50 Mo | Négligeable |
| **Total app (stack optimisée)** | **~1.4-1.5 Go** | ✅ Compatible 6-8 Go RAM |

**Économie vs spec initiale :** -500 Mo (LLM 1.5B → 0.5B) +300 Mo (Whisper small → large-v3-turbo) = **net -200 Mo et meilleure qualité**

### Projection latence finale (stack optimisée)

```
Raccourci pressé
    ↓
Capture audio (cpal)
    ↓
TEN VAD détecte fin de parole                    ~50ms
    ↓
Whisper large-v3-turbo Q5_0
  - Encoder CoreML/ANE                           ~150ms
  - Décodeur Metal                               ~300ms
  Total STT                                      ~450-600ms
    ↓
Règles locales (toujours)                        < 1ms
Score confiance élevé → Coller direct
Sinon → Qwen2.5-0.5B Q4                         ~200-300ms
    ↓
Paste au curseur (Accessibility API Swift)       ~10ms

TOTAL (M2/M3, cas avec LLM)                     ~700-950ms ✅
TOTAL (M2/M3, cas sans LLM)                     ~500-650ms ✅
TOTAL (M1, cas avec LLM)                        ~1.0-1.4s  ✅
```

**→ Objectif < 1.5s atteint sur tous les Mac M-series avec la stack optimisée.**

**Stratégie de gestion mémoire :**
- Chargement à la demande : les modèles ne sont chargés que quand l'utilisateur active la dictée
- Déchargement après timeout d'inactivité (configurable, comme Handy)
- Option "décharger immédiatement" pour machines à 6 Go
- Possibilité de ne charger QUE le STT (sans LLM) pour un mode ultra-léger

**Workflow de validation (avant fine-tuning) :**
1. Baseline Whisper large-v3-turbo seul → mesurer le zero-edit rate natif en FR
2. Si zero-edit rate > 70% sans LLM → le LLM devient optionnel/mode qualité uniquement
3. Si baseline 50-70% → fine-tuning SFT LoRA sur 5-10k paires FR

## Implementation Plan

### Étape 1 — Foundation : Fork Handy + nettoyage

- [ ] **Task 1 : Fork et rebrand**
  - File: repo GitHub (fork depuis https://github.com/cjpais/Handy)
  - Action: Forker le repo, renommer en `dictation-ia-locale`, adapter `Cargo.toml` et `tauri.conf.json` (nom app, identifiants bundle)
  - Notes: Conserver la structure Handy intacte pour le MVP, pas de réarchitecture prématurée

- [ ] **Task 2 : Nettoyage du codebase Handy**
  - Files: `src-tauri/Cargo.toml`, `src-tauri/src/managers/transcription.rs`, `src-tauri/src/managers/model.rs`
  - Action: Retirer les dépendances Parakeet / Moonshine / SenseVoice de `transcribe-rs`. Supprimer le code conditionnel Windows/Linux (macOS only pour MVP). Retirer `enigo` (remplacé par Swift Accessibility).
  - Notes: Conserver `cpal`, `rubato`, `rdev`, `vad-rs` tels quels dans un premier temps

---

### Étape 2 — STT : Remplacement transcribe-rs → whisper.cpp CoreML

- [ ] **Task 3 : Build whisper.cpp avec CoreML + Metal**
  - File: `build.rs` ou script `scripts/build-whisper.sh`
  - Action: Configurer la compilation de whisper.cpp en tant que dépendance native : `cmake -B build -DWHISPER_COREML=1 -DWHISPER_METAL=1`. Générer les modèles CoreML (script Python `generate_coreml_model.py` fourni par whisper.cpp).
  - Notes: Nécessite Xcode + CMake installés. Le modèle CoreML compilé (~800 Mo) devra être inclus dans le bundle app ou téléchargé au premier lancement.

- [ ] **Task 4 : whisper_ffi.rs — bindings Rust → whisper.cpp**
  - File: `src-tauri/src/whisper_ffi.rs`
  - Action: Créer les bindings unsafe Rust vers l'API C de whisper.cpp. Exposer : `whisper_init_from_file()`, `whisper_full()`, `whisper_full_get_segment_text()`, `whisper_free()`.
  - Notes: Utiliser `bindgen` ou bindings manuels. Wrapper safe en Rust autour des appels unsafe.

- [ ] **Task 5 : managers/transcription.rs — adapter pour whisper.cpp FFI**
  - File: `src-tauri/src/managers/transcription.rs`
  - Action: Remplacer les appels `transcribe-rs` par l'utilisation de `whisper_ffi.rs`. Configurer les paramètres FR optimisés : `language: "fr"`, `beam_size: 1`, `best_of: 1`, `temperature: 0.0`, `no_speech_threshold: 0.6`. Récupérer le score de confiance Whisper pour le routing pipeline.
  - Notes: Conserver la logique de chargement/déchargement du modèle à la demande (pattern Handy)

- [ ] **Task 6 : Swift plugin — WhisperANE.swift (CoreML ANE encoder)**
  - File: `src-tauri/swift-plugin/WhisperANE.swift`
  - Action: Créer le plugin Swift qui expose le CoreML encoder de Whisper sur l'Apple Neural Engine. Implémenter la FFI Rust↔Swift via `@_cdecl` pour appel depuis Rust. L'encoder Swift traite le mel spectrogram, le décodeur reste côté whisper.cpp/Metal.
  - Notes: Alternative V2 : utiliser directement WhisperKit (argmaxinc/WhisperKit) qui gère le pipeline complet CoreML. Pour le MVP, l'approche encoder-seul via FFI est plus légère.

---

### Étape 3 — VAD : Benchmark et intégration

- [ ] **Task 7 : Benchmark TEN VAD vs Silero v4**
  - File: `tests/vad_benchmark.rs`
  - Action: Créer un test de benchmark qui compare la latence de détection fin-de-parole entre `vad-rs` (Silero v4) et `ten-vad` sur des enregistrements FR de 5-15s. Mesurer : latence switch speech→silence, faux positifs, faux négatifs.
  - Notes: Si TEN VAD n'a pas de binding Rust natif, évaluer wrapping C. Gain attendu : -100 à -200ms sur la détection de fin de parole.

- [ ] **Task 8 : Intégration VAD retenu dans audio_toolkit/**
  - File: `src-tauri/src/audio_toolkit/vad.rs`
  - Action: Intégrer le VAD retenu (TEN VAD si benchmark favorable, sinon Silero v4). Conserver les paramètres SmoothedVad de Handy (prefill 15 frames, hangover 15 frames, onset 2 frames) comme base, ajuster selon les résultats.
  - Notes: Le VAD déclenche la fin de l'enregistrement → directement relié au pipeline

---

### Étape 4 — Pipeline hybride : Règles + LLM conditionnel

- [ ] **Task 9 : pipeline/rules.rs — nettoyage local < 1ms**
  - File: `src-tauri/src/pipeline/rules.rs`
  - Action: Implémenter les règles de nettoyage FR sans LLM : (1) Filler words FR : suppression de "euh", "heu", "bah", "ben", "du coup", "genre", "voilà", "quoi", "en fait", "eh bien". (2) Collapse des bégaiements (répétitions consécutives du même mot). (3) Majuscule en début de phrase. (4) Point final si absent.
  - Notes: Implémenter comme pipeline de regex compilées (zéro allocation à l'exécution). Benchmarker < 1ms sur une phrase de 100 tokens.

- [ ] **Task 10 : llm/cleanup.rs — intégration Qwen2.5-0.5B Q4**
  - File: `src-tauri/src/llm/cleanup.rs`
  - Action: Intégrer `llama-cpp-rs` pour charger et exécuter Qwen2.5-0.5B-Instruct Q4. Implémenter : `load_model()`, `cleanup_text(raw: &str, mode: Mode) -> String`, `unload_model()`. Paramètres : `n_predict: 128, temperature: 0.0, top_k: 1, repeat_penalty: 1.0`.
  - Notes: Chargement à la demande + déchargement après 5min d'inactivité (configurable). RAM ~300 Mo quand chargé.

- [ ] **Task 11 : pipeline/modes.rs — prompts Chat/Pro/Code**
  - File: `src-tauri/src/pipeline/modes.rs`
  - Action: Définir les 3 prompts système par mode :
    - **Chat** : "Corrige uniquement l'orthographe et ajoute la ponctuation de base. Conserve le ton et la structure. Ne reformule pas."
    - **Pro** : "Reformule ce texte de manière concise et professionnelle pour un email ou document. Paragraphes clairs, ton poli."
    - **Code** : "Corrige la ponctuation. Préserve tous les termes techniques anglais, identifiants et symboles. Ne traduis jamais le jargon technique."
  - Notes: Prompts en français, incluent le texte brut transcrit, retournent uniquement le texte nettoyé (pas d'explication)

- [ ] **Task 12 : pipeline/orchestrator.rs — routing confiance**
  - File: `src-tauri/src/pipeline/orchestrator.rs`
  - Action: Implémenter l'orchestrateur pipeline avec routing conditionnel via channels Tokio :
    1. Reçoit : (audio_buffer, mode, confidence_score)
    2. Appelle `rules::apply(text)` (toujours, < 1ms)
    3. Si `confidence_score >= 0.85` ET `word_count <= 30` ET mode Chat → retourne le texte nettoyé par règles
    4. Sinon → appelle `cleanup::run(text, mode)` (LLM)
    5. Retourne le texte final vers le collage
  - Notes: Les seuils (0.85, 30 mots) sont configurables. Logguer la décision (règles vs LLM) pour analyse ultérieure.

---

### Étape 5 — macOS native : Swift plugins

- [ ] **Task 13 : MenuBar.swift — NSStatusBar natif**
  - File: `src-tauri/swift-plugin/MenuBar.swift`
  - Action: Remplacer la menu bar Tauri par NSStatusBar natif : icône micro dans la barre de menu, animation pendant l'enregistrement (pulse), statuts (idle / recording / processing). Exposer via FFI Rust.
  - Notes: NSStatusBar natif = comportement macOS correct (opacity au clic, position, taille d'icône). Le menu contextuel (settings, quit) aussi natif.

- [ ] **Task 14 : AccessibilityPaste.swift — collage direct au curseur**
  - File: `src-tauri/swift-plugin/AccessibilityPaste.swift`
  - Action: Implémenter le collage au curseur via AXUIElement (Accessibility API macOS) sans simuler Cmd+V : copier dans le presse-papier puis appel AXUIElement pour coller directement dans l'app active. Exposer via FFI Rust.
  - Notes: Nécessite l'autorisation Accessibility dans System Preferences. Fallback : Cmd+V via CGEvent si AXUIElement échoue. Tester dans Chrome, VSCode, Notion, Mail, Slack.

- [ ] **Task 15 : Raccourcis clavier globaux (adaptation Handy)**
  - File: `src-tauri/src/shortcut/mod.rs`
  - Action: Conserver la logique `rdev` de Handy pour les raccourcis globaux. Adapter : configurer les raccourcis par défaut (ex: Fn double-tap ou Option+Space), ajouter support Mode Rapide (raccourci 1) vs Mode Qualité (raccourci 2, optionnel).
  - Notes: Le raccourci global fonctionne dans toutes les apps sans focus. Debounce 30ms (valeur Handy) conservée.

---

### Étape 6 — Capture audio (adaptation Handy)

- [ ] **Task 16 : managers/audio.rs — adaptation cpal**
  - File: `src-tauri/src/managers/audio.rs`
  - Action: Adapter le code audio de Handy : capture `cpal` en f32 mono → resampling `rubato` vers 16kHz → buffer circulaire pour envoi au VAD. Conserver la gestion des périphériques audio de Handy.
  - Notes: Quasi identique à Handy — modifications minimes (nettoyage code Windows/Linux, ajout commentaires FR)

---

### Étape 7 — Benchmark baseline + tests

- [ ] **Task 17 : tests/benchmark.rs — latence + zero-edit rate**
  - File: `tests/benchmark.rs`
  - Action: Créer un benchmark automatisé : (1) Charge 100 phrases FR variées (corpus fixtures audio WAV). (2) Mesure la latence p50/p99 pour chaque étape du pipeline. (3) Compare le texte transcrit vs référence → calcule le zero-edit rate. Sortie : rapport CSV + résumé terminal.
  - Notes: À lancer à chaque commit via CI (GitHub Actions). Les fixtures audio seront des enregistrements de test de 5-15s en FR.

- [ ] **Task 18 : Benchmark baseline — Whisper turbo seul (sans LLM)**
  - File: `tests/benchmark.rs` (exécution)
  - Action: Lancer le benchmark avec `orchestrator.rs` configuré en mode "règles seulement" (LLM désactivé). Mesurer le zero-edit rate natif de Whisper large-v3-turbo en FR.
  - Notes: **Décision clé** : Si zero-edit rate > 70% → le LLM devient optionnel (Mode Qualité uniquement). Si 50-70% → fine-tuning nécessaire. Documenter le résultat dans `docs/dev-log.md`.

---

### Étape 8 — UI Settings (React/TypeScript)

- [ ] **Task 19 : src/components/Settings.tsx — UI configuration**
  - File: `src/components/Settings.tsx`, `src/App.tsx`
  - Action: Adapter l'UI settings de Handy pour nos besoins : (1) Sélecteur de mode d'écriture (Chat / Pro / Code). (2) Configuration des raccourcis clavier. (3) Toggle LLM on/off (mode léger). (4) Indicateur de statut modèles (chargé / déchargé / téléchargement). (5) Historique des dernières transcriptions.
  - Notes: Conserver le style React/TypeScript de Handy. Minimal : pas de redesign complet pour le MVP.

---

### Étape 9 — Fine-tuning LLM (post-MVP, après benchmark baseline)

- [ ] **Task 20 : Construction dataset 10k paires FR**
  - File: `training/dataset/build_dataset.py`
  - Action: Assembler 10k paires (transcription brute → texte propre) :
    - 3k : Common Voice FR v18+ → Whisper small (brut) / transcription humaine (propre)
    - 3k : MLS FR (même méthode)
    - 2k : ESLO/TCOF (ORTOLANG) — français spontané
    - 2k : Génération synthétique LLM (homophones FR : ces/ses/c'est, à/a, ou/où, et/est)
  - Notes: Télécharger Common Voice FR depuis https://commonvoice.mozilla.org/fr/datasets. ESLO/TCOF via ORTOLANG (inscription requise).

- [ ] **Task 21 : Fine-tuning LoRA/QLoRA Qwen2.5-0.5B**
  - File: `training/finetune.py`
  - Action: Fine-tuner Qwen2.5-0.5B-Instruct avec LoRA (r=16, alpha=32) sur le dataset de 10k paires. Évaluer le zero-edit rate sur le set de test (500 paires réservées). Quantiser le modèle fine-tuné en GGUF Q4_K_M pour llama.cpp.
  - Notes: Hardware requis : GPU 8+ Go VRAM (RTX 3070 / A100 / RunPod). Frameworks : Hugging Face Transformers + PEFT + Unsloth (2x plus rapide). Comparer le zero-edit rate avant/après fine-tuning.

- [ ] **Task 22 : Intégration modèle fine-tuné**
  - File: `src-tauri/src/llm/cleanup.rs`
  - Action: Remplacer le modèle Qwen2.5-0.5B base par la version fine-tunée. Valider la latence (doit rester ~200-300ms). Mettre à jour le script de téléchargement du modèle.
  - Notes: Le modèle fine-tuné GGUF Q4_K_M doit être hébergé (GitHub Releases ou Hugging Face Hub) pour le téléchargement au premier lancement.

### Acceptance Criteria

- [ ] **AC-01** : Given un utilisateur FR qui parle 10s dans son micro, when il active la dictée via raccourci clavier, then le texte transcrit apparaît en < 2s avec un WER < 10% sur des phrases standard FR (objectif stretch : < 1.5s sur M2/M3)

- [ ] **AC-02** : Given une transcription brute avec fillers FR ("euh", "du coup", "genre"), when le pipeline de règles la traite, then les fillers sont supprimés et la ponctuation de base ajoutée en < 1ms

- [ ] **AC-03** : Given une transcription brute avec score de confiance Whisper >= 0.85 et <= 30 mots en mode Chat, when le pipeline s'exécute, then le texte est retourné par les règles seules (sans LLM) en < 650ms total

- [ ] **AC-04** : Given la même phrase dictée traitée en mode Chat / Pro / Code, when on compare les 3 sorties, then le style est distinctement différent : Chat (conversationnel préservé), Pro (reformulé concis), Code (jargon technique intact)

- [ ] **AC-05** : Given l'application avec Whisper large-v3-turbo + Qwen2.5-0.5B tous deux chargés, when on mesure la RAM de l'app, then elle reste < 1.6 Go (compatible machines 6-8 Go RAM)

- [ ] **AC-06** : Given une machine en mode avion (wifi désactivé), when l'utilisateur utilise toutes les fonctionnalités (dictée, LLM, modes), then tout fonctionne sans erreur ni dégradation

- [ ] **AC-07** : Given le curseur positionné dans Chrome / VSCode / Notion / Mail / Slack, when la transcription est terminée, then le texte est collé à la position exacte du curseur sans avoir à changer d'app

- [ ] **AC-08** : Given un benchmark lancé sur 100 phrases FR avec Whisper large-v3-turbo seul (sans LLM), when on mesure le zero-edit rate, then il est documenté (seuil de décision : > 70% = LLM optionnel, < 70% = fine-tuning requis)

- [ ] **AC-09** : Given le pipeline complet (avec LLM activé) sur un M1/M2/M3, when on mesure la latence p99 sur 50 enregistrements, then elle est < 1.4s sur M1 et < 950ms sur M2/M3

- [ ] **AC-10** : Given n'importe quelle application en focus, when l'utilisateur appuie sur le raccourci global, then l'enregistrement démarre avec feedback visuel (icône menu bar animée) et sonore (bip)

## Additional Context

### Dependencies

| Dépendance | Version | Rôle | Source |
|------------|---------|------|--------|
| Tauri | 2.x | Framework desktop | crates.io |
| whisper.cpp (via FFI) | latest | STT Whisper CoreML + Metal | github.com/ggerganov/whisper.cpp |
| llama-cpp-rs | latest | LLM Qwen2.5-0.5B via llama.cpp | crates.io |
| ten-vad | latest | Voice Activity Detection (benchmark vs vad-rs) | github.com/TEN-framework/ten-vad |
| vad-rs | latest | Silero VAD v4 (fallback si TEN VAD non concluant) | crates.io (Handy) |
| cpal | latest | Capture audio | crates.io |
| rdev | latest | Raccourcis clavier globaux | crates.io |
| rubato | latest | Resampling audio | crates.io |
| tokio | latest | Async runtime + channels pipeline | crates.io |
| **Modèle STT** | large-v3-turbo Q5_0 | ~800 Mo | huggingface.co/ggerganov/whisper.cpp |
| **Modèle LLM** | Qwen2.5-0.5B-Instruct Q4 | ~300 Mo | huggingface.co/Qwen |
| **Swift (plugin)** | — | ANE + NSStatusBar + AXUIElement | macOS SDK |

> `transcribe-rs` et `enigo` retirés — remplacés par whisper.cpp FFI direct et Accessibility API Swift

### Prérequis pour le fine-tuning LLM

- **Hardware :** GPU avec 8+ Go VRAM (ou cloud GPU type RunPod/Vast.ai)
- **Framework :** Hugging Face Transformers + PEFT (LoRA) ou Unsloth
- **Dataset :** 5 000-10 000 paires (transcription brute FR → texte propre FR)
- **Sources de données possibles :**
  - Enregistrements personnels transcrits et corrigés manuellement
  - Datasets open source de correction grammaticale FR (ex: WiKEd-FR, FCGEC)
  - Transcriptions YouTube FR nettoyées
  - Génération synthétique via un gros LLM (Claude/GPT) qui simule des erreurs de transcription

### Testing Strategy

**Tests unitaires Rust (cargo test) :**
- `pipeline/rules.rs` : test sur 50 phrases FR avec fillers, bégaiements, ponctuation manquante
- `llm/cleanup.rs` : test chargement/déchargement modèle, timeout inactivité
- `pipeline/orchestrator.rs` : test routing conditionnel (règles vs LLM selon score confiance)
- `whisper_ffi.rs` : test bindings unsafe (smoke test sur audio fixture court)

**Tests d'intégration :**
- Pipeline complet : audio WAV fixture FR 5s → transcription → règles → texte final (assert WER < 10%)
- Test collage dans 5 applications : Chrome, VSCode, Notion, Mail, Slack (tests manuels)

**Benchmarks automatisés (tests/benchmark.rs) :**
- Latence p50/p99 par étape : VAD / STT / Règles / LLM / Paste
- Zero-edit rate baseline (Whisper turbo seul) sur 100 phrases FR
- Zero-edit rate post-fine-tuning (comparaison avant/après)
- À lancer à chaque commit via GitHub Actions

**Tests manuels :**
- Dictée de 5min en FR dans différentes applications (email, Slack, code, documents)
- Test sur machine 6 Go RAM (pression mémoire maximale)
- Test raccourcis globaux dans toutes les apps (Chrome, VS Code, terminal, Finder)
- Test mode avion (vérification 100% offline)

### Notes

**Risques techniques (pre-mortem) :**

1. **FFI Rust↔Swift** : Complexité des bindings. Mitigation : utiliser `@_cdecl` Swift + `extern "C"` Rust, démarrer par un "hello world" FFI avant d'implémenter la logique. Alternative : tout garder en Rust si Swift FFI bloque.

2. **Zero-edit rate LLM baseline** : Résultat incertain avant le benchmark. Si > 70% sans LLM → revoir l'architecture (LLM optionnel seulement). Si < 50% → fine-tuning urgent et dataset plus grand nécessaire.

3. **Modèle CoreML Whisper** : La génération des fichiers CoreML (`generate_coreml_model.py`) nécessite Python + coremltools + Xcode. Peut prendre 20-40min pour large-v3-turbo. À faire en amont du développement.

4. **Distribution app macOS** : Notarisation Apple requise pour distribuer hors App Store. Les permissions Accessibility et microphone doivent être déclarées dans `entitlements.plist`. Prévoir ce travail avant la release MVP.

5. **Fine-tuning dataset** : Construire 10k paires FR de qualité = 2-4 semaines de travail. Mitigation : démarrer avec modèle base + bons prompts, fine-tuner en itération. ESLO/TCOF requiert une inscription (délai possible).

**Décisions à prendre après le benchmark baseline (Task 18) :**
- Si zero-edit rate Whisper seul > 70% : le LLM passe en Mode Qualité uniquement (optionnel par défaut)
- Si 50-70% : fine-tuning obligatoire pour atteindre l'objectif
- Si < 50% : revoir les prompts, évaluer Qwen2.5-1.5B comme fallback

**Roadmap V2 (post-MVP) :**
- Migration full Swift natif + CoreML (abandon Tauri, ~30 Mo RAM vs ~200 Mo)
- WhisperKit (argmaxinc) pour ANE complet (encoder + décodeur)
- Streaming STT (chunking pendant l'enregistrement, -200-400ms latence)
- Modèle LLM CoreML via ANE (2-3x plus rapide que llama.cpp Metal)
- App Store distribution (notarisation simplifiée en Swift natif)
