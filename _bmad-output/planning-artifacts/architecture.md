---
stepsCompleted: ['architecture-complete']
inputDocuments:
  - '{output_folder}/planning-artifacts/prd.md'
  - 'implementation-artifacts/tech-spec-dictation-ia-locale-mvp.md'
  - 'docs/technical-review-final.md'
  - 'docs/specs/functional-spec.md'
  - 'docs/adr/001-backend-language.md'
workflowType: 'architecture'
project_name: 'DictAI'
date: '2026-02-25'
status: 'complete'
---

# Document d'Architecture -- DictAI

## 1. Vue d'ensemble

### 1.1 Description du systeme

DictAI est une application de dictee vocale intelligente, **locale par defaut**, concue specifiquement pour le francais. L'utilisateur parle naturellement dans son microphone ; DictAI transcrit, nettoie et formate le texte en temps reel, puis le colle automatiquement au curseur actif dans n'importe quelle application.

Le produit fonctionne comme une **menu bar app macOS** : aucune fenetre dans le dock, activation par raccourci clavier global, retour visuel par overlay et icone dans la barre de menu.

### 1.2 Philosophie fondatrice

| Principe | Implication architecturale |
|----------|---------------------------|
| **Local-first, privacy-first** | Zero envoi de donnees vocales au cloud. Whisper + LLM tournent 100% sur la machine. |
| **Francophone natif** | Regles de nettoyage FR compilees (filler words, elisions, ponctuation), prompts LLM en francais, modele Whisper force en `language: "fr"`. |
| **Fluidite percue** | Pipeline hybride : regles regex < 1ms pour 60-65% des cas, LLM conditionnel uniquement quand necessaire. |
| **Simplicite utilisateur** | Zero-config au premier lancement. Pas de compte, pas de configuration Whisper manuelle. |
| **Open-core** | Desktop gratuit open-source (MIT). Synchronisation cross-device en option payante (4,99 EUR/mois). |

### 1.3 Contraintes techniques cles

- **Budget RAM** : < 4 Go pour l'application complete (Whisper + LLM charges), compatible machines 6-8 Go
- **Latence pipeline** : p95 < 3s en mode rules-only, p95 < 5s avec LLM (objectif stretch : < 1.5s sur M2/M3)
- **Taille app** : < 2 Go installee (modeles inclus)
- **Plateforme MVP** : macOS uniquement (Apple Silicon prioritaire, Intel en fallback)
- **100% offline** : aucune connexion reseau requise pour le pipeline core

### 1.4 Heritage : fork de Handy

DictAI est un fork de **Handy** (github.com/cjpais/Handy, 15k+ stars, licence MIT). L'architecture Rust + Tauri 2.x est conservee comme base, enrichie d'un pipeline de post-traitement LLM local, de regles de nettoyage FR natives, et de plugins Swift pour les composants macOS critiques.

---

## 2. Decisions architecturales (ADR)

### ADR-001 -- Backend : Rust via fork Handy / Tauri 2.x

| Champ | Valeur |
|-------|--------|
| **Statut** | Accepte |
| **Date** | 2026-02-24 |
| **Contexte** | Besoin d'un backend performant pour un pipeline audio/ML temps reel. Options evaluees : Python, Swift natif, Rust. |
| **Decision** | Rust via fork de Handy (Tauri 2.x). |
| **Raison** | Performance native, empreinte memoire minimale, ecosysteme audio/ML Rust mature (vad-rs, cpal, rubato). Le fork Handy economise des semaines de dev (pipeline audio, raccourcis, collage deja implementes). Tauri 2.x = ~10 Mo vs ~150 Mo Electron. |
| **Trade-off** | Courbe d'apprentissage Rust. WebView Tauri = ~200 Mo RAM (vs ~30 Mo pour du Swift natif pur). |
| **Alternatives rejetees** | Python (trop lent pour le pipeline temps reel), Swift natif pur (perte du fork Handy, plus long a implementer pour le MVP). |
| **Evolution V2** | Migration complete vers Swift natif + CoreML (abandon Tauri, gain ~170 Mo RAM). |

### ADR-002 -- STT : Whisper large-v3-turbo Q5_0 via whisper.cpp FFI (CoreML + Metal)

| Champ | Valeur |
|-------|--------|
| **Statut** | Accepte |
| **Date** | 2026-02-24 |
| **Contexte** | Le modele STT doit offrir un WER francais < 10%, tourner localement en < 600ms, et tenir dans ~800 Mo de RAM. |
| **Decision** | Whisper `large-v3-turbo` quantise Q5_0 (~800 Mo) via `whisper.cpp` compile avec `WHISPER_COREML=1` + `WHISPER_METAL=1`, appele depuis Rust via FFI C. |
| **Raison (unanime 3 sources)** | Decodeur reduit de 32 a 4 couches = 5.4x plus rapide que large-v3. WER francais 3-8% (vs 12-15% pour small). Ponctuation et majuscules gerees nativement, reduisant la charge du LLM. CoreML encoder sur ANE = 3x plus rapide. Metal decoder = 3-4x plus rapide. |
| **Abandon de transcribe-rs** | whisper.cpp directement via FFI Rust donne acces natif aux optimisations CoreML encoder (ANE) + Metal decoder. transcribe-rs ajoutait une abstraction sans gain. |
| **Parametres optimises** | `language: "fr"`, `beam_size: 1`, `best_of: 1`, `temperature: 0.0`, `no_speech_threshold: 0.6` |
| **Latence cible** | ~450-600ms (Encoder ANE ~150ms + Decodeur Metal ~300ms) |
| **Budget RAM** | ~800 Mo pour le modele charge |
| **Alternative future** | Faster-Whisper (CTranslate2) si binding Rust disponible -- potentiellement < 300ms |

### ADR-003 -- LLM nettoyage : Qwen2.5-0.5B Q4 via Ollama HTTP

| Champ | Valeur |
|-------|--------|
| **Statut** | Accepte |
| **Date** | 2026-02-24 |
| **Contexte** | Le LLM de nettoyage doit corriger/reformuler des transcriptions FR en < 300ms avec < 400 Mo de RAM. |
| **Decision** | `Qwen2.5-0.5B-Instruct Q4` via Ollama HTTP (`http://127.0.0.1:11434/api/chat`). |
| **Raison (unanime 3 sources)** | La tache est etroite (correction, non generation creative). Un modele 0.5B fine-tune atteint > 70% zero-edit rate. RAM ~300 Mo vs ~900 Mo pour 1.5B. |
| **Implementation MVP** | Ollama HTTP (reqwest::blocking dans un thread dedie, timeout 8s). Simplicite d'integration, gestion des modeles par Ollama. |
| **Evolution post-MVP** | Migration vers `llama-cpp-rs` (integration directe sans processus Ollama externe) ou CoreML ANE en V2. |
| **Parametres** | `n_predict: 128`, `temperature: 0.0`, `top_k: 1`, `repeat_penalty: 1.0` |
| **Fallback** | Qwen2.5-1.5B Q4 si le 0.5B s'avere insuffisant apres benchmark. Si Ollama absent : mode rules-only automatique. |

### ADR-004 -- Fine-tuning LLM pour le nettoyage FR

| Champ | Valeur |
|-------|--------|
| **Statut** | Planifie (post-MVP) |
| **Date** | 2026-02-24 |
| **Contexte** | Le modele Qwen2.5-0.5B de base pourrait ne pas atteindre un zero-edit rate suffisant en francais sans fine-tuning. |
| **Decision** | Fine-tuning supervise (SFT) avec QLoRA sur 10k paires FR. |
| **Dataset** | 3k Common Voice FR + 3k MLS FR + 2k ESLO/TCOF (francais spontane) + 2k synthetique (homophones FR). |
| **Outils** | Hugging Face Transformers + PEFT (LoRA r=16, alpha=32) + Unsloth. Google Colab ou RunPod (GPU 8+ Go VRAM). |
| **Validation** | Zero-edit rate sur set de test 500 phrases FR. Seuil : > 70% pour valider le modele fine-tune. |
| **Prerequis** | Benchmark baseline (ADR-009) doit etre realise avant : si zero-edit rate > 70% sans LLM, le fine-tuning devient optionnel. |

### ADR-005 -- Framework desktop : Tauri 2.x

| Champ | Valeur |
|-------|--------|
| **Statut** | Accepte |
| **Date** | 2026-02-24 |
| **Decision** | Conserver Tauri 2.x (herite de Handy). |
| **Raison** | Leger (~10 Mo pour le runtime, WebView natif macOS), backend Rust natif, deja integre dans le fork Handy. Frontend React/TypeScript pour les settings. |
| **UI** | Menu bar macOS natif (via plugin Swift NSStatusBar) + fenetre settings React/TypeScript. |
| **Evolution V2** | Migration vers SwiftUI natif (abandon WebView, gain ~170 Mo RAM). |

### ADR-006 -- Plateforme cible : macOS d'abord, mobile separe

| Champ | Valeur |
|-------|--------|
| **Statut** | Accepte |
| **Date** | 2026-02-24 |
| **Decision** | MVP macOS uniquement. Android (Kotlin natif) post-MVP. iOS (Swift natif) futur. |
| **Raison** | Tauri ne supporte pas Android nativement. Un MVP desktop solide avant de fragmenter l'effort. La cible primaire (developpeurs, redacteurs, freelances) est majoritairement sur macOS. |
| **Android** | Runtime separe en Kotlin + whisper.cpp/llama.cpp via NDK. |
| **iOS** | Swift natif + CoreML. Partage potentiel de code avec la V2 desktop Swift. |

### ADR-007 -- Architecture hybride : Rust + Plugin Swift via FFI

| Champ | Valeur |
|-------|--------|
| **Statut** | Accepte |
| **Date** | 2026-02-24 |
| **Contexte** | Certains composants macOS critiques (ANE, menu bar native, collage curseur) sont uniquement accessibles via des APIs Swift/Objective-C. |
| **Decision** | Couche de plugins Swift exposes via `@_cdecl` pour FFI depuis Rust. |
| **Composants Swift** | `WhisperANE.swift` (encoder CoreML sur ANE), `MenuBar.swift` (NSStatusBar natif), `AccessibilityPaste.swift` (AXUIElement API pour collage direct). |
| **Ce qui reste en Rust** | Orchestration pipeline, VAD, LLM cleanup, async Tokio, historique SQLite, raccourcis clavier, gestion audio. |
| **FFI** | `@_cdecl("function_name")` cote Swift, `extern "C"` cote Rust. Types C simples (pointeurs, entiers, chaines C). |
| **Trade-off** | Complexite FFI Rust-Swift, mais gains significatifs en latence (ANE) et UX (menu bar native, collage sans Cmd+V). |

### ADR-008 -- VAD : Silero v4 (TEN VAD en evaluation)

| Champ | Valeur |
|-------|--------|
| **Statut** | Accepte (Silero v4 en production, TEN VAD en benchmark) |
| **Date** | 2026-02-24 |
| **Decision** | `vad-rs` (Silero v4) comme baseline. Benchmark `ten-vad` (TEN-framework) en cours. Adoption si gain confirme. |
| **Implementation actuelle** | SmoothedVad wrapper autour de Silero v4 : prefill 15 frames, hangover 15 frames, onset 2 frames, seuil 0.3. |
| **Performance mesuree** | p99 = 170 microsecondes. Largement sous la contrainte < 1ms. |
| **Gain attendu TEN VAD** | -100 a -200ms sur la detection de fin de parole (latence switch speech-silence reduite). |

### ADR-009 -- Pipeline hybride : regles locales + LLM conditionnel

| Champ | Valeur |
|-------|--------|
| **Statut** | Accepte et implemente |
| **Date** | 2026-02-24 |
| **Contexte** | Appeler le LLM systematiquement ajouterait 200-300ms a chaque dictee. La plupart des phrases courtes/simples n'en ont pas besoin. |
| **Decision** | Pipeline en deux etapes avec routing conditionnel. |
| **Regles locales (toujours appliquees, < 1ms)** | Filler words FR (euh, heu, bah, ben, du coup, genre, voila, quoi, en fait, eh bien, hein, bref, etc.), collapse begaiements, normalisation elisions Whisper (j' ai -> j'ai), ponctuation doublee, majuscule debut + point final. |
| **Seuils de routing** | `confidence >= 0.82` ET `word_count <= 30` ET `mode != Pro` -> regles seules. Sinon -> regles + LLM. |
| **Resultat attendu** | 60-65% des cas sans LLM (< 5ms total post-traitement). LLM pour 35-40% (~200-300ms). |
| **Performance mesuree** | Rules p99 = 415 microsecondes. 35/35 tests pipeline FR passent. |

---

## 3. Architecture systeme

### 3.1 Diagramme de couches

```
+=====================================================================+
|                        COUCHE PRESENTATION                          |
|  +--------------------------+  +-------------------------------+    |
|  |  Frontend React/TS       |  |  Menu Bar Swift (NSStatusBar) |    |
|  |  - Settings (3 sections) |  |  - Icone micro (idle/rec/proc)|   |
|  |  - WriteModeSelector     |  |  - Menu contextuel natif      |    |
|  |  - Overlay enregistrement|  |  - Animation pulse            |    |
|  |  - Onboarding permissions|  +-------------------------------+    |
|  |  - Historique             |                                      |
|  |  - Debug panel            |                                      |
|  +--------------------------+                                       |
+=====================================================================+
|                         COUCHE IPC (Tauri)                          |
|  Commandes #[tauri::command] + Evenements Tauri (emit/listen)       |
+=====================================================================+
|                        COUCHE BACKEND RUST                          |
|  +------------------+  +------------------+  +------------------+   |
|  |  Shortcut Manager|  |  Audio Manager   |  |  Pipeline        |   |
|  |  - rdev global   |  |  - cpal capture  |  |  - orchestrator  |   |
|  |  - debounce 30ms |  |  - rubato 16kHz  |  |  - rules (FR)    |   |
|  |  - push-to-talk  |  |  - SmoothedVad   |  |  - modes         |   |
|  |  - toggle mode   |  |  - buffer PCM    |  |  - routing       |   |
|  +------------------+  +------------------+  +------------------+   |
|  +------------------+  +------------------+  +------------------+   |
|  |  Transcription   |  |  LLM Cleanup     |  |  History Manager |   |
|  |  - whisper_ffi   |  |  - Ollama HTTP   |  |  - SQLite        |   |
|  |  - WhisperParams |  |  - reqwest::block|  |  - CRUD sessions |   |
|  |  - WhisperResult |  |  - timeout 8s    |  |  - horodatage    |   |
|  +------------------+  +------------------+  +------------------+   |
|  +------------------+  +------------------+                         |
|  |  Model Manager   |  |  Settings        |                        |
|  |  - download      |  |  - tauri-store   |                        |
|  |  - load/unload   |  |  - AppSettings   |                        |
|  |  - timeout inact.|  |  - ShortcutBind  |                        |
|  +------------------+  +------------------+                         |
+=====================================================================+
|                     COUCHE SWIFT PLUGINS (FFI)                      |
|  +------------------+  +------------------+  +------------------+   |
|  | WhisperANE.swift |  | MenuBar.swift    |  | AccessibilityPaste|  |
|  | - ANE detection  |  | - NSStatusBar    |  | - AXUIElement     |  |
|  | - CoreML encoder |  | - icone animee   |  | - fallback Cmd+V  |  |
|  | @_cdecl FFI      |  | @_cdecl FFI      |  | @_cdecl FFI       |  |
|  +------------------+  +------------------+  +------------------+   |
+=====================================================================+
|                    COUCHE SYSTEME (macOS)                           |
|  Microphone | Accessibility API | ANE/CoreML | Metal | Clipboard   |
+=====================================================================+
```

### 3.2 Composants principaux

| Composant | Responsabilite | Langage | Crate/Framework |
|-----------|---------------|---------|-----------------|
| **Shortcut Manager** | Capture raccourcis clavier globaux, debounce, modes toggle/push-to-talk | Rust | `rdev` |
| **Audio Manager** | Capture micro cpal, resampling 16kHz, buffer PCM, mute optionnel | Rust | `cpal`, `rubato` |
| **VAD** | Detection debut/fin de parole, lissage temporel (SmoothedVad) | Rust | `vad-rs` (Silero v4) |
| **Transcription** | Inferences Whisper via FFI, gestion parametres FR | Rust + C | `whisper.cpp` (FFI) |
| **Pipeline** | Orchestration, regles FR, modes d'ecriture, routing conditionnel | Rust | custom |
| **LLM Cleanup** | Post-traitement via Ollama HTTP, prompts par mode | Rust | `reqwest` |
| **History** | Persistence des sessions de dictee (texte brut, traite, mode, timestamp) | Rust | `rusqlite` |
| **Model Manager** | Telechargement, chargement/dechargement a la demande, timeout inactivite | Rust | custom |
| **Settings** | Configuration persistante (raccourcis, micro, mode, LLM toggle, debug) | Rust | `tauri-plugin-store` |
| **WhisperANE** | Detection ANE, encoder CoreML sur Apple Neural Engine | Swift | CoreML |
| **MenuBar** | Icone menu bar native macOS, animation, menu contextuel | Swift | NSStatusBar |
| **AccessibilityPaste** | Collage au curseur via AXUIElement, fallback Cmd+V | Swift | ApplicationServices |
| **Frontend** | Interface settings, overlay, onboarding, historique | TypeScript | React, Tailwind |

### 3.3 Flux de donnees principal

```
Utilisateur appuie sur raccourci
        |
        v
[rdev] intercepte le keydown global
        |
        v
[Audio Manager] demarre capture cpal (f32 mono)
        |
        v
[rubato] resampling -> 16kHz PCM
        |
        v
[SmoothedVad] analyse frames en continu
        |
        |-- parole detectee -> continue capture
        |-- silence detecte (hangover expire) -> stop capture
        v
Buffer audio PCM complet
        |
        v
[whisper_ffi] inference Whisper large-v3-turbo Q5_0
  |-- [WhisperANE.swift] encoder CoreML sur ANE (~150ms)
  |-- [whisper.cpp] decodeur Metal GPU (~300ms)
        |
        v
WhisperResult { text, no_speech_prob }
        |
        v
[Pipeline Orchestrator] routing conditionnel
  |-- confidence >= 0.82 ET words <= 30 ET mode != Pro
  |     -> [rules.rs] nettoyage regex (< 1ms)
  |     -> texte final
  |
  |-- sinon
        -> [rules.rs] nettoyage regex (< 1ms)
        -> [cleanup.rs] Ollama HTTP Qwen2.5-0.5B (~200-300ms)
        -> texte final
        |
        v
[AccessibilityPaste.swift] collage au curseur
  |-- tentative 1 : AXUIElement (direct, ~2ms)
  |-- tentative 2 : presse-papier + CGEvent Cmd+V (~50ms)
        |
        v
[History Manager] sauvegarde SQLite (texte brut, traite, mode, timestamp)
        |
        v
[Tauri event] -> Frontend met a jour overlay/historique
```

---

## 4. Pipeline de dictee

### 4.1 Etapes detaillees

| Etape | Composant | Temps cible | Description |
|-------|-----------|-------------|-------------|
| 1. Activation | `rdev` shortcut | instantane | Raccourci global capture, debounce 30ms |
| 2. Feedback | MenuBar.swift + audio | < 10ms | Animation icone + son stylo sur papier |
| 3. Capture | `cpal` + `rubato` | duree parole | f32 mono -> 16kHz PCM, buffer circulaire |
| 4. VAD | `SmoothedVad` (Silero v4) | ~0.2ms/frame | Prefill 15, hangover 15, onset 2, seuil 0.3 |
| 5. Fin parole | VAD hangover expire | ~50ms | Detection fin de parole |
| 6. STT Encoder | WhisperANE.swift (CoreML) | ~150ms | Mel spectrogram -> features (ANE) |
| 7. STT Decoder | whisper.cpp (Metal) | ~300ms | Features -> tokens texte FR |
| 8. Regles FR | `rules.rs` | < 1ms (p99 415us) | Filler words, elisions, ponctuation, begaiements |
| 9. Routing | `orchestrator.rs` | < 0.1ms | Decision rules-only vs rules+LLM |
| 10. LLM (conditionnel) | `cleanup.rs` via Ollama | ~200-300ms | Correction/reformulation selon mode |
| 11. Collage | AccessibilityPaste.swift | ~2-50ms | AXUIElement ou fallback Cmd+V |
| 12. Historique | `rusqlite` | < 5ms | Sauvegarde session complete |

### 4.2 Logique de routing hybride

```
                     Transcription Whisper
                            |
                            v
                    [Regles FR < 1ms]
                    (toujours appliquees)
                            |
                            v
              +-------------+-------------+
              |                           |
     confidence >= 0.82              confidence < 0.82
     ET words <= 30                  OU words > 30
     ET mode != Pro                  OU mode == Pro
              |                           |
              v                           v
       Texte final               [LLM Qwen2.5-0.5B]
       (regles seules)            ~200-300ms
       ~500-650ms total                   |
                                          v
                                   Texte final
                                   ~700-950ms total
```

**Seuils configurables dans `orchestrator.rs` :**
- `CONFIDENCE_THRESHOLD` : 0.82 (confiance Whisper minimale pour eviter le LLM)
- `MAX_WORDS_FAST_PATH` : 30 (nombre max de mots pour le fast-path)

### 4.3 Modes d'ecriture

| Mode | Comportement | LLM obligatoire | Prompt systeme |
|------|-------------|-----------------|----------------|
| **Chat** (defaut) | Correction orthographe minimale, ponctuation basique, ton conserve | Non (si confiance haute) | "Corrige uniquement l'orthographe evidente et ajoute la ponctuation de base. Conserve exactement le ton et la structure originale. Ne reformule pas." |
| **Pro** | Reformulation concise et professionnelle, paragraphes clairs | **Oui (toujours)** | "Reformule ce texte transcrit de maniere concise et professionnelle, adapte pour un email ou document." |
| **Code** | Jargon technique preserve, identifiants intacts, Markdown | Non (si confiance haute) | "Corrige la ponctuation. Preserve TOUS les termes techniques anglais, identifiants, symboles. Ne traduis jamais le jargon technique." |

### 4.4 Regles FR implementees (rules.rs)

Les regles sont compilees en regex statiques via `once_cell::sync::Lazy` (zero allocation a l'execution) :

| Regle | Pattern | Remplacement |
|-------|---------|-------------|
| Filler words FR | `euh+, heu+, bah, bon ben, ben, du coup, genre, voila, quoi, en fait, eh bien, hein, bref, tu vois, vous voyez, a vrai dire, en gros, disons que, si tu veux, en quelque sorte, en tout cas, n'est-ce pas, pas vrai, ouais bon, pfff, ah bon` | suppression |
| Elisions Whisper | `j' ai, c' est, n' est, l' homme, d' accord, qu' il` | `j'ai, c'est, n'est, l'homme, d'accord, qu'il` |
| Ponctuation doublee | `..` -> `.`, `...+` -> `...`, `??` -> `?`, `!!` -> `!`, `,,` -> `,` | normalisation |
| Begaiements | `je je je veux` | `je veux` (collapse par comparaison case-insensitive) |
| Majuscule initiale | debut de phrase sans majuscule | ajout majuscule |
| Point final | phrase sans ponctuation terminale | ajout point |
| Espaces multiples | `mot  mot` | `mot mot` |

---

## 5. Architecture des donnees

### 5.1 Schema SQLite (historique)

```sql
-- Table principale des sessions de dictee
CREATE TABLE IF NOT EXISTS transcriptions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    raw_text    TEXT NOT NULL,       -- Texte brut Whisper (avant regles/LLM)
    clean_text  TEXT NOT NULL,       -- Texte final (apres pipeline complet)
    mode        TEXT NOT NULL,       -- 'chat' | 'pro' | 'code'
    route       TEXT NOT NULL,       -- 'rules_only' | 'rules_and_llm'
    confidence  REAL,               -- Score confiance Whisper [0.0, 1.0]
    duration_ms INTEGER,            -- Duree audio enregistree (ms)
    pipeline_ms INTEGER,            -- Latence pipeline totale (ms)
    word_count  INTEGER             -- Nombre de mots du texte final
);

-- Index pour consultation historique
CREATE INDEX IF NOT EXISTS idx_transcriptions_date ON transcriptions(created_at DESC);
```

### 5.2 Stockage settings

Les parametres sont persistes via `tauri-plugin-store` dans un fichier JSON :

**Emplacement :** `~/Library/Application Support/DictAI/settings.json`

**Structure `AppSettings` :**

```rust
pub struct AppSettings {
    // Raccourcis
    pub shortcut_bindings: Vec<ShortcutBinding>,

    // Audio
    pub selected_microphone: Option<String>,
    pub mute_while_recording: bool,

    // Pipeline
    pub write_mode: WriteMode,          // Chat | Pro | Code
    pub llm_enabled: bool,              // Toggle LLM on/off
    pub confidence_threshold: f32,      // Seuil routing (defaut 0.82)

    // Modeles
    pub whisper_model_path: Option<String>,
    pub model_unload_timeout_secs: u64, // Timeout dechargement (defaut 300s)

    // UI
    pub log_level: LogLevel,
    pub play_sounds: bool,
    pub show_overlay: bool,
    pub auto_start: bool,

    // Debug
    pub debug_mode: bool,
}
```

### 5.3 Stockage modeles

**Emplacement :** `~/Library/Application Support/DictAI/models/`

| Modele | Fichier | Taille |
|--------|---------|--------|
| Whisper large-v3-turbo Q5_0 | `ggml-large-v3-turbo-q5_0.bin` | ~800 Mo |
| CoreML encoder (optionnel) | `ggml-large-v3-turbo-encoder.mlpackage/` | ~400 Mo |
| Qwen2.5-0.5B Q4 | Gere par Ollama (`~/.ollama/models/`) | ~300 Mo |

---

## 6. Couche Swift (plugins macOS)

### 6.1 Architecture FFI Rust - Swift

```
Rust (src-tauri/src/)              Swift (src-tauri/swift-plugin/)
+---------------------+           +-------------------------+
| whisper_ffi.rs      |  extern C | WhisperANE.swift        |
|   whisper_ane_      |---------->|   @_cdecl("whisper_     |
|   available()       |           |   ane_available")       |
+---------------------+           +-------------------------+

+---------------------+           +-------------------------+
| clipboard.rs        |  extern C | AccessibilityPaste.swift|
|   accessibility_    |---------->|   @_cdecl("accessibility|
|   paste_text()      |           |   _paste_text")         |
+---------------------+           +-------------------------+

+---------------------+           +-------------------------+
| tray.rs             |  extern C | MenuBar.swift           |
|   menubar_set_      |---------->|   @_cdecl("menubar_    |
|   state()           |           |   set_state")           |
+---------------------+           +-------------------------+
```

**Convention FFI :**
- Cote Swift : `@_cdecl("nom_fonction")` expose la fonction avec linkage C
- Cote Rust : `extern "C" { fn nom_fonction(...) -> ...; }` declare le binding
- Types partages : `*const c_char`, `c_int`, `Bool` (types C simples uniquement)
- Les chaines sont passees comme `*const CChar` et converties via `String(cString:)`

### 6.2 WhisperANE.swift

**Fonctionnalites :**
1. **Detection ANE** : `whisper_ane_available()` retourne `true` sur Apple Silicon (arch arm64)
2. **Encoder CoreML** : Charge `ggml-large-v3-turbo-encoder.mlpackage`, execute l'encodeur Whisper sur ANE (~150ms vs ~500ms GPU seul)

**Prerequis :**
```bash
pip3 install coremltools openai-whisper
python3 vendor/whisper.cpp/models/generate_coreml_model.py large-v3-turbo
```

### 6.3 AccessibilityPaste.swift

**Strategie a 2 niveaux de fallback :**

| Niveau | Methode | Latence | Compatibilite |
|--------|---------|---------|---------------|
| 1 | AXUIElement `kAXSelectedTextAttribute` | ~2ms | Apps Cocoa natives (TextEdit, Notes, Mail) |
| 2 | Presse-papier + CGEvent Cmd+V | ~50ms | Universelle (Chrome, VS Code, Slack, Notion) |

Le fallback sauvegarde/restaure le contenu du presse-papier pour ne pas ecraser le contenu precedent de l'utilisateur.

**Permission requise :** `AXIsProcessTrusted()` -- Accessibility dans System Settings.

### 6.4 MenuBar.swift

**Fonctionnalites :**
- Icone micro dans NSStatusBar (pas la menu bar Tauri artificielle)
- 3 etats visuels : idle (icone statique), recording (pulse vert), processing (spinner)
- Menu contextuel natif : mode d'ecriture, ouvrir settings, quitter
- Comportement macOS correct (opacity au clic, position, taille d'icone)

---

## 7. Communication Frontend - Backend

### 7.1 Commandes Tauri IPC

Les commandes sont exposees via `#[tauri::command]` avec types generes par `specta` :

| Commande | Direction | Description |
|----------|-----------|-------------|
| `get_app_settings` | Frontend -> Backend | Lecture de la configuration complete |
| `get_default_settings` | Frontend -> Backend | Valeurs par defaut de la configuration |
| `set_log_level` | Frontend -> Backend | Changement du niveau de log |
| `cancel_operation` | Frontend -> Backend | Annulation de l'operation en cours |
| `initialize_enigo` | Frontend -> Backend | Initialisation du systeme d'input clavier |
| `initialize_shortcuts` | Frontend -> Backend | Initialisation des raccourcis apres permissions |
| `check_apple_intelligence_available` | Frontend -> Backend | Retourne toujours `false` (desactive) |
| `get_app_dir_path` | Frontend -> Backend | Chemin du repertoire Application Support |
| `get_log_dir_path` | Frontend -> Backend | Chemin du repertoire de logs |
| `open_recordings_folder` | Frontend -> Backend | Ouvre le dossier d'enregistrements dans Finder |
| `open_log_dir` | Frontend -> Backend | Ouvre le dossier de logs dans Finder |
| `open_app_data_dir` | Frontend -> Backend | Ouvre le repertoire de donnees dans Finder |

**Commandes audio (commands/audio.rs) :**
- `get_audio_devices` -- Liste les peripheriques audio disponibles
- `start_recording` -- Demarre la capture micro
- `stop_recording` -- Arrete la capture

**Commandes transcription (commands/transcription.rs) :**
- `transcribe` -- Lance l'inference Whisper sur le buffer audio
- `get_transcription_status` -- Etat courant du pipeline

**Commandes historique (commands/history.rs) :**
- `get_history` -- Lecture paginee de l'historique
- `delete_history_entry` -- Suppression d'une entree
- `clear_history` -- Suppression complete

**Commandes modeles (commands/models.rs) :**
- `get_available_models` -- Liste les modeles telecharges
- `download_model` -- Telecharge un modele depuis le hub
- `get_model_status` -- Etat du modele (charge/decharge/telechargement)

### 7.2 Evenements Tauri

| Evenement | Direction | Payload |
|-----------|-----------|---------|
| `recording-started` | Backend -> Frontend | `{ binding_id: String }` |
| `recording-stopped` | Backend -> Frontend | `{ duration_ms: u64 }` |
| `transcription-progress` | Backend -> Frontend | `{ step: String, percent: f32 }` |
| `transcription-complete` | Backend -> Frontend | `{ text: String, route: String, pipeline_ms: u64 }` |
| `model-download-progress` | Backend -> Frontend | `{ model: String, percent: f32 }` |
| `model-loaded` | Backend -> Frontend | `{ model: String }` |
| `model-unloaded` | Backend -> Frontend | `{ model: String }` |
| `error` | Backend -> Frontend | `{ message: String, code: String }` |

### 7.3 Synchronisation des settings

Le flux de synchronisation settings est unidirectionnel :
1. Le frontend lit les settings via `get_app_settings`
2. Les modifications frontend sont envoyees via des commandes dediees (ex: `set_log_level`)
3. Le backend persiste via `tauri-plugin-store`
4. Le backend emet un evenement de confirmation
5. Le frontend rafraichit son etat local

---

## 8. Gestion des modeles

### 8.1 Cycle de vie des modeles

```
                    ABSENT
                      |
                      v
              [download_model]
                      |
                      v
                  DOWNLOADED
                      |
                      v
                [load_model]
                      |
                      v
                   LOADED ---------> [timeout inactivite]
                      |                      |
                      v                      v
                 (inference)            UNLOADED
                      |                      |
                      v                      v
                   LOADED <--------- [load_model]
```

### 8.2 Strategie de chargement

| Strategie | Description | Usage |
|-----------|-------------|-------|
| **A la demande** | Le modele se charge au premier raccourci appuye (pendant que l'utilisateur parle) | Defaut |
| **Pre-charge** | Le modele Whisper est charge au demarrage de l'app | Option avancee |
| **Dechargement timeout** | Apres N secondes d'inactivite, le modele est decharge | Defaut : 300s (5min) |
| **Dechargement immediat** | Le modele est decharge des la fin de chaque dictee | Option pour machines 6 Go |

### 8.3 Telechargement initial

Au premier lancement, si le modele Whisper n'est pas present :
1. Ecran d'onboarding avec explication en francais
2. Barre de progression du telechargement (~800 Mo)
3. Verification d'integrite (hash SHA256)
4. Stockage dans `~/Library/Application Support/DictAI/models/`

Pour Ollama (LLM) :
1. Detection de la presence d'Ollama (`http://127.0.0.1:11434/api/tags`)
2. Si absent : message clair "Installer Ollama pour activer le mode Pro" + lien
3. Si present : verification du modele `qwen2.5:0.5b`, suggestion `ollama pull` si absent
4. Mode rules-only automatique si Ollama indisponible

---

## 9. Budget memoire (RAM)

### 9.1 Ventilation detaillee

| Composant | RAM estimee | Notes |
|-----------|------------|-------|
| macOS + apps arriere-plan | ~3-4 Go | Incompressible, hors perimetre app |
| **Tauri runtime + WebView** | **~200 Mo** | WebView macOS inclus |
| **Whisper large-v3-turbo Q5_0** | **~800 Mo** | Modele charge en memoire |
| **CoreML encoder cache** | **~100 Mo** | Cache d'inference ANE (estime) |
| **Qwen2.5-0.5B Q4 (via Ollama)** | **~300 Mo** | Processus Ollama separe |
| **Buffer audio + VAD** | **~50 Mo** | Buffers PCM + modele Silero (~2 Mo) |
| **SQLite + settings** | **~10 Mo** | Negligeable |
| **Rust runtime + tokio** | **~40 Mo** | Threads + channels async |
| **Total application** | **~1.5 Go** | Compatible machines 6-8 Go |

### 9.2 Modes de fonctionnement RAM

| Mode | RAM app | Cible |
|------|---------|-------|
| **Complet** (Whisper + LLM) | ~1.5 Go | Machines 8+ Go |
| **Leger** (Whisper seul, LLM off) | ~1.1 Go | Machines 6 Go |
| **Idle** (modeles decharges) | ~250 Mo | Au repos apres timeout |

### 9.3 Comparaison avec la vision V2

| Composant | V1 (Rust/Tauri) | V2 (Swift natif) |
|-----------|----------------|-------------------|
| Runtime UI | ~200 Mo (WebView) | ~30 Mo (SwiftUI) |
| STT | ~800 Mo (whisper.cpp) | ~800 Mo (WhisperKit CoreML) |
| LLM | ~300 Mo (Ollama) | ~300 Mo (CoreML ANE) |
| **Total** | **~1.5 Go** | **~1.2 Go** |

---

## 10. Projection de latence

### 10.1 Ventilation par etape

| Etape | M1 | M2 | M3/M4 | Notes |
|-------|-----|-----|--------|-------|
| VAD fin de parole | ~50ms | ~50ms | ~50ms | SmoothedVad hangover |
| Whisper encoder (ANE) | ~200ms | ~150ms | ~120ms | CoreML sur ANE |
| Whisper decodeur (Metal) | ~400ms | ~300ms | ~250ms | GPU Metal |
| **Total STT** | **~600ms** | **~450ms** | **~370ms** | |
| Regles FR | < 1ms | < 1ms | < 1ms | p99 = 415us |
| LLM Qwen (conditionnel) | ~300ms | ~250ms | ~200ms | Ollama local |
| Collage curseur | ~10ms | ~10ms | ~10ms | AXUIElement |
| **Total sans LLM** | **~660ms** | **~510ms** | **~430ms** | 60-65% des cas |
| **Total avec LLM** | **~960ms** | **~760ms** | **~630ms** | 35-40% des cas |

### 10.2 Objectifs PRD vs projections

| Metrique | Objectif PRD | Projection M1 | Projection M2/M3 |
|----------|-------------|---------------|-------------------|
| Latence p95 (rules-only, 15 mots) | < 3s | ~1.0s | ~0.7s |
| Latence p95 (avec LLM, 15 mots) | < 5s | ~1.4s | ~1.0s |
| Latence stretch goal | < 1.5s | ~1.0s | ~0.7s |

Les projections depassent largement les objectifs PRD. Le stretch goal < 1.5s est atteignable sur tous les Mac M-series.

### 10.3 Goulots d'etranglement identifies

1. **Cold start modele** : Premier chargement Whisper ~2-5s. Mitigation : pre-chargement au demarrage ou pendant que l'utilisateur parle (pattern Handy conserve).
2. **Cold start Ollama** : Premier appel LLM ~1-2s (modele pas en memoire). Mitigation : `ollama serve` au demarrage + keep-alive configurable.
3. **Phrases longues (> 30 mots)** : Force le chemin LLM meme en mode Chat. Mitigation : seuil configurable.

---

## 11. Strategie de test

### 11.1 Tests unitaires Rust (`cargo test`)

| Module | Couverture | Description |
|--------|-----------|-------------|
| `pipeline/rules.rs` | 35 tests | Filler words FR, elisions, ponctuation doublee, begaiements, majuscules, points finals, espaces |
| `pipeline/modes.rs` | 3 tests | FromStr, prompts non-vides, `always_use_llm` pour Pro |
| `pipeline/orchestrator.rs` | 5+ tests | Routing conditionnel (seuils, modes, edge cases) |
| `whisper_ffi.rs` | smoke test | Bindings unsafe sur audio fixture court |
| `llm/cleanup.rs` | 3+ tests | Construction payload, parsing reponse, timeout |

### 11.2 Tests d'integration

| Test | Description | Methode |
|------|-------------|---------|
| Pipeline complet | Audio WAV fixture FR 5s -> transcription -> regles -> texte final | Automatise (`cargo test`) |
| Collage multi-app | Collage dans Chrome, VSCode, Notion, Mail, Slack | Manuel |
| Mode avion | Verification 100% offline (wifi desactive) | Manuel |
| Pression memoire | Test sur machine 6 Go RAM | Manuel |
| Raccourcis globaux | Raccourcis dans toutes les apps | Manuel |

### 11.3 Benchmarks automatises (`tests/benchmark.rs`)

| Benchmark | Metrique | Seuil |
|-----------|---------|-------|
| Latence VAD | p99 | < 1ms (mesure : 170us) |
| Latence rules | p99 | < 1ms (mesure : 415us) |
| Latence STT | p50, p99 | < 600ms M2/M3 |
| Latence LLM | p50, p99 | < 300ms M2/M3 |
| Latence pipeline complet | p50, p99 | < 950ms M2/M3 avec LLM |
| Zero-edit rate | pourcentage | > 70% (seuil decision fine-tuning) |

Les benchmarks sont conçus pour tourner sur un corpus de 100 phrases FR avec fixtures audio WAV.

### 11.4 CI/CD (GitHub Actions)

- `cargo check` et `cargo test` a chaque push
- Benchmark de latence et zero-edit rate a chaque merge sur `main`
- Resultats sauvegardes en CSV pour suivi historique

---

## 12. Securite et vie privee

### 12.1 Principes de securite

| Principe | Implementation |
|----------|---------------|
| **Zero donnee au cloud** | Aucun appel reseau dans le pipeline core. Whisper et LLM tournent localement. |
| **Pas de telemetrie par defaut** | Opt-in explicite pour analytics anonymisees (compteurs d'usage uniquement). |
| **Pas de credentials en dur** | NFR-11. Aucune cle API, token ou secret dans le code source. |
| **Donnees sous controle utilisateur** | Historique SQLite dans `~/Library/Application Support/DictAI/`. L'utilisateur peut supprimer a tout moment. |
| **Audio ephemere** | Le buffer audio PCM est libere apres transcription. Aucun fichier audio persiste par defaut. |

### 12.2 Conformite RGPD

| Aspect | Statut |
|--------|--------|
| Base legale | Interet legitime (traitement 100% local, pas de transfert) |
| Droit d'acces | L'utilisateur a un acces direct a toutes ses donnees (SQLite local) |
| Droit a l'effacement | Suppression historique via l'interface ou suppression du fichier SQLite |
| Droit a la portabilite | Export historique (SQLite standard, lisible par tout outil) |
| Information utilisateur | Onboarding en francais expliquant le traitement 100% local |
| Consentement analytics | Opt-in explicite, jamais par defaut |

### 12.3 Permissions macOS requises

| Permission | Usage | Declaration |
|------------|-------|-------------|
| **Microphone** (`NSMicrophoneUsageDescription`) | Capture audio pour transcription vocale | `Info.plist` |
| **Accessibility** (`AXIsProcessTrusted`) | Raccourci clavier global + collage au curseur via AXUIElement | Demande via dialog systeme |

Les permissions sont demandees lors de l'onboarding avec des explications claires en francais, sans jargon technique. Si refusees, l'application explique pourquoi elle ne peut pas fonctionner et guide l'utilisateur vers System Settings.

### 12.4 Securite du build

| Mesure | Description |
|--------|-------------|
| Code signing | Certificat developpeur Apple pour distribution hors App Store |
| Notarization | Soumission a Apple pour verification (requis macOS Ventura+) |
| Entitlements | `com.apple.security.device.microphone`, `com.apple.security.accessibility` |
| Integrite modeles | Verification SHA256 apres telechargement |

---

## 13. Plan de deploiement

### 13.1 Distribution

| Canal | Format | Notes |
|-------|--------|-------|
| **GitHub Releases** | `.dmg` (universal binary) | Canal principal. Code signe + notarise. |
| **Homebrew Cask** | `brew install --cask dictai` | Post-MVP, si adoption communautaire suffisante. |
| **Mac App Store** | `.app` sandboxe | Futur (V2 Swift natif simplifie la soumission). |

### 13.2 Processus de release

```
1. Tag git semver (vX.Y.Z)
        |
        v
2. GitHub Actions CI
   - cargo test (unitaires + integration)
   - cargo build --release (universal binary arm64 + x86_64)
   - Benchmark latence / zero-edit rate
        |
        v
3. Code signing
   - Signature avec certificat Developer ID Application
   - Timestamp serveur Apple
        |
        v
4. Notarization
   - Soumission a Apple (xcrun notarytool)
   - Attente validation (~5-15min)
   - Agrafage du ticket (xcrun stapler)
        |
        v
5. Packaging
   - Creation .dmg avec fond personnalise (theme "cahier de classe")
   - Inclusion ou non du modele Whisper (2 variantes possibles)
        |
        v
6. Publication
   - Upload sur GitHub Releases
   - Mise a jour du endpoint Tauri Updater (JSON manifest)
   - Annonce (GitHub Discussions, Reddit r/france, r/devfr)
```

### 13.3 Auto-update

- **Mecanisme** : Tauri Updater verifie un endpoint JSON sur GitHub Releases au lancement
- **Telechargement** : En arriere-plan, silencieux
- **Application** : Au prochain redemarrage de l'app
- **Modeles** : Geres separement (pas dans l'updater app). Le modele Whisper n'est telecharge qu'au premier lancement ou lors d'une mise a jour majeure du modele.

### 13.4 Versioning

- **Application** : Semver `MAJOR.MINOR.PATCH` (ex: `1.0.0`, `1.1.0`, `1.1.1`)
- **Modeles** : Versiones par nom de modele (ex: `large-v3-turbo-q5_0`)
- **Base de donnees** : Migrations SQLite via `rusqlite_migration` (schema versione)

---

## 14. Evolution V2

### 14.1 Vision : migration Swift natif + CoreML

La V2 represente un pivot architectural majeur : abandon de Tauri/Rust au profit d'une application 100% Swift native avec CoreML pour tous les modeles ML.

```
V1 (MVP actuel)                         V2 (cible 12+ mois)
+-------------------+                   +-------------------+
| React/TS WebView  | -> supprime ->    | SwiftUI natif     |
| Tauri 2.x runtime | -> supprime ->    | (pas de WebView)  |
| Rust backend      | -> supprime ->    |                   |
| whisper.cpp FFI   | -> remplace ->    | WhisperKit CoreML |
| Ollama HTTP       | -> remplace ->    | CoreML ANE (LLM)  |
| vad-rs Silero     | -> conserve ->    | vad-rs ou CoreML  |
| Swift plugins FFI | -> integre ->     | Swift natif direct|
+-------------------+                   +-------------------+
```

### 14.2 Gains attendus V2

| Aspect | V1 (Rust/Tauri) | V2 (Swift natif) | Gain |
|--------|----------------|-------------------|------|
| RAM au repos | ~250 Mo | ~30 Mo | **-220 Mo** |
| RAM en fonctionnement | ~1.5 Go | ~1.2 Go | **-300 Mo** |
| Latence LLM | llama.cpp Metal | CoreML ANE | **2-3x plus rapide** |
| Latence STT | whisper.cpp + CoreML encoder | WhisperKit pipeline complet ANE | **potentiellement < 300ms** |
| Menu bar | NSStatusBar (via FFI) | NSStatusBar (natif) | Simplicite |
| Collage | AXUIElement (via FFI) | AXUIElement (natif) | Simplicite |
| Distribution | Notarization manuelle | App Store possible | Simplicite |
| Taille binaire | ~30 Mo + WebView | ~15 Mo | **-50%** |

### 14.3 Feuille de route V2

| Phase | Composant | Description |
|-------|-----------|-------------|
| V2-alpha | WhisperKit | Remplacement whisper.cpp par WhisperKit (argmaxinc). Pipeline CoreML complet (encoder + decodeur sur ANE). |
| V2-alpha | SwiftUI | Remplacement du frontend React par SwiftUI natif. Suppression de la dependance WebView. |
| V2-beta | CoreML LLM | Migration du LLM de Ollama vers un modele CoreML sur ANE. Suppression de la dependance Ollama. |
| V2-beta | Streaming | Transcription par chunks pendant l'enregistrement (chunking 1-2s). Gain -200 a -400ms de latence percue. |
| V2-rc | Consolidation | Suppression complete du code Rust. App 100% Swift. Tests de regression sur le corpus FR. |

### 14.4 Plateformes post-MVP

| Plateforme | Stack | Timeline |
|------------|-------|----------|
| **Android** | Kotlin natif + whisper.cpp/llama.cpp via NDK | M+6 |
| **iOS** | Swift natif + CoreML (partage de code avec V2 desktop) | M+12 |
| **Windows** | Tauri (reutilisation V1 Rust) si demande communautaire | Post M+12 |
| **Linux** | Tauri (reutilisation V1 Rust) si demande communautaire | Post M+12 |

### 14.5 Synchronisation cross-device (Supabase)

- **Modele** : 4,99 EUR/mois (seul composant payant)
- **Backend** : Supabase (free tier initial)
- **Donnees synchronisees** : Historique, settings, vocabulaire personnel
- **Auth** : Supabase Auth (email/password, Magic Link)
- **Chiffrement** : Donnees chiffrees en transit (HTTPS) et au repos (Supabase RLS)

---

## 15. Risques techniques et mitigations

### 15.1 Risques identifies (pre-mortem)

| # | Risque | Probabilite | Impact | Mitigation |
|---|--------|-------------|--------|------------|
| R1 | **FFI Rust-Swift instable** : Les bindings `@_cdecl` peuvent causer des crashes memoire si les types C sont mal geres | Moyenne | Eleve | Utiliser des types C simples (int, char*). Smoke tests FFI systematiques. Alternative : tout garder en Rust si Swift FFI bloque. |
| R2 | **Zero-edit rate LLM insuffisant** : Le modele Qwen2.5-0.5B de base pourrait ne pas atteindre 70% de zero-edit rate en francais | Moyenne | Moyen | Benchmark baseline sans LLM d'abord. Si > 70% avec Whisper seul, le LLM devient optionnel. Sinon : fine-tuning QLoRA sur 10k paires FR. Fallback : Qwen2.5-1.5B. |
| R3 | **Generation CoreML lente** : Le script `generate_coreml_model.py` prend 20-40min et necessite Python + coremltools + Xcode | Faible | Moyen | Generer le modele CoreML une seule fois et le distribuer pre-compile. L'inclure dans le bundle ou le telecharger au premier lancement. |
| R4 | **Utilisateur sans Ollama** : Le LLM necessite Ollama installe et en cours d'execution | Elevee | Moyen | Fallback rules-only automatique et transparent. Message clair dans l'onboarding. Guide d'installation Ollama en 2 etapes. |
| R5 | **Mac Intel < 16 Go RAM** : Lenteur ou crash par pression memoire | Moyenne | Eleve | Detection hardware au demarrage. Warning pour Intel. Suggestion mode leger (LLM off). Modele Whisper plus petit en fallback si detection < 8 Go RAM. |
| R6 | **Permissions macOS refusees** : L'app est non fonctionnelle sans Microphone et Accessibility | Elevee | Critique | Onboarding dedie avec explication de chaque permission en francais. Detection continue du statut des permissions. Guide de reactivation si refusees. |
| R7 | **Audio ambiant bruyant** : Transcription degradee dans des environnements bruyants | Moyenne | Moyen | VAD stricte (seuil configurable). Message "environnement bruyant detecte" si `no_speech_prob` eleve. |
| R8 | **Notarization Apple** : Processus requis pour macOS Ventura+ mais peut echouer si les entitlements sont mal configures | Moyenne | Eleve | Integrer la notarization en CI/CD des le debut. Tester sur Ventura, Sonoma, Sequoia. Certificat Developer ID requis (~99 USD/an). |
| R9 | **Regression qualite pipeline** : Les mises a jour du modele Whisper ou des regles peuvent degrader la qualite | Faible | Moyen | Benchmark automatise a chaque commit (100 phrases FR). Seuils de regression definis. Test de regression obligatoire avant release. |
| R10 | **Cold start Ollama** : Premier appel LLM ~1-2s si le modele n'est pas en memoire Ollama | Moyenne | Faible | Commande `ollama serve` au demarrage de l'app. Keep-alive configurable dans Ollama. Pre-warmup au lancement (appel muet). |
| R11 | **Dataset fine-tuning** : Construire 10k paires FR de qualite = 2-4 semaines. ESLO/TCOF requiert une inscription (delai possible) | Moyenne | Moyen | Demarrer avec modele base + bons prompts. Fine-tuner en iteration. Common Voice FR (acces libre) en priorite. Generation synthetique LLM pour completer. |
| R12 | **Conflit raccourcis clavier** : D'autres apps peuvent utiliser le meme raccourci global | Faible | Faible | Detection de conflits au demarrage. Raccourci configurable par l'utilisateur. Raccourci par defaut choisi pour minimiser les conflits (Cmd+Shift+D). |

### 15.2 Matrice de decision post-benchmark

Le benchmark baseline (Whisper turbo seul, sans LLM) determine l'architecture finale :

| Resultat zero-edit rate | Decision |
|------------------------|----------|
| > 70% sans LLM | Le LLM passe en "Mode Qualite" optionnel. Le mode Chat utilise les regles seules par defaut. Economie de RAM et latence. |
| 50-70% sans LLM | Fine-tuning QLoRA obligatoire sur 10k paires FR. Le LLM reste conditionnel selon les seuils de routing. |
| < 50% sans LLM | Revoir les prompts et regles. Evaluer Qwen2.5-1.5B comme fallback. Considerer un modele Whisper plus gros ou fine-tune Whisper. |

---

## Annexe A : Dependances Rust completes

| Crate | Version | Role | Source |
|-------|---------|------|--------|
| `tauri` | 2.9.1 | Framework desktop | crates.io |
| `cpal` | 0.16 | Capture audio cross-platform | crates.io |
| `vad-rs` | git | Silero VAD v4 | github.com/cjpais/vad-rs |
| `rubato` | 0.16 | Resampling audio | crates.io |
| `rodio` | git | Feedback audio (sons) | github.com/cjpais/rodio |
| `rdev` | git | Raccourcis clavier globaux | github.com/rustdesk-org/rdev |
| `reqwest` | 0.12 | HTTP client (Ollama API) | crates.io |
| `rusqlite` | 0.37 | SQLite (historique) | crates.io |
| `tokio` | 1.x | Async runtime + channels | crates.io |
| `regex` | 1.x | Regles de nettoyage FR | crates.io |
| `once_cell` | 1.x | Lazy static (regex compilees) | crates.io |
| `anyhow` | 1.x | Gestion d'erreurs | crates.io |
| `serde` / `serde_json` | 1.x | Serialisation | crates.io |
| `chrono` | 0.4 | Horodatage | crates.io |
| `hound` | 3.5 | Lecture/ecriture WAV | crates.io |
| `transcribe-rs` | 0.2.5 | STT Whisper (temporaire, remplace par whisper_ffi) | crates.io |
| `enigo` | 0.6.1 | Simulation clavier (temporaire, remplace par Swift) | crates.io |
| `llama-cpp-2` | 0.1 | LLM via llama.cpp (optionnel, non utilise MVP) | crates.io |
| `whisper.cpp` | native | STT via FFI C | vendor/ |

## Annexe B : Structure du code source

```
dictation-ia-locale/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs                    -- Point d'entree Tauri
│   │   ├── lib.rs                     -- Library crate
│   │   ├── settings.rs                -- AppSettings, ShortcutBinding, LogLevel
│   │   ├── whisper_ffi.rs             -- Bindings Rust -> whisper.cpp (FFI C)
│   │   ├── actions.rs                 -- Logique start/stop transcription
│   │   ├── input.rs                   -- EnigoState (simulation clavier)
│   │   ├── overlay.rs                 -- Gestion overlay visuel
│   │   ├── clipboard.rs              -- Presse-papier + bridge FFI Swift
│   │   ├── tray.rs                    -- Icone menu bar Tauri
│   │   ├── tray_i18n.rs              -- Traductions menu bar
│   │   ├── audio_feedback.rs          -- Sons (activation/desactivation)
│   │   ├── llm_client.rs             -- Client HTTP generique LLM
│   │   ├── pipeline/
│   │   │   ├── mod.rs
│   │   │   ├── orchestrator.rs        -- Routing conditionnel (ADR-009)
│   │   │   ├── rules.rs              -- Regles FR compilees (35 tests)
│   │   │   └── modes.rs              -- WriteMode enum + prompts
│   │   ├── llm/
│   │   │   ├── mod.rs
│   │   │   └── cleanup.rs            -- Ollama HTTP (Qwen2.5-0.5B)
│   │   ├── managers/
│   │   │   ├── audio.rs              -- Capture micro (cpal + SmoothedVad)
│   │   │   ├── transcription.rs      -- Pipeline STT (whisper_ffi)
│   │   │   ├── model.rs              -- Download/load/unload modeles
│   │   │   └── history.rs            -- CRUD SQLite historique
│   │   ├── audio_toolkit/
│   │   │   ├── vad/                   -- SmoothedVad, SileroVad
│   │   │   └── audio/                -- Device, Recorder, Resampler
│   │   ├── shortcut/
│   │   │   ├── mod.rs                -- Init raccourcis
│   │   │   ├── handler.rs            -- Callback raccourcis
│   │   │   ├── tauri_impl.rs         -- Integration Tauri
│   │   │   └── handy_keys.rs         -- Mappings touches (Handy)
│   │   └── commands/
│   │       ├── mod.rs                -- Commandes IPC generales
│   │       ├── audio.rs              -- Commandes audio
│   │       ├── transcription.rs      -- Commandes transcription
│   │       ├── history.rs            -- Commandes historique
│   │       └── models.rs             -- Commandes modeles
│   ├── swift-plugin/
│   │   ├── WhisperANE.swift           -- CoreML encoder ANE + detection
│   │   ├── MenuBar.swift              -- NSStatusBar natif
│   │   └── AccessibilityPaste.swift   -- AXUIElement collage curseur
│   ├── Cargo.toml
│   └── build.rs                       -- Build whisper.cpp + flags
├── src/
│   ├── App.tsx
│   ├── components/
│   │   ├── Sidebar.tsx
│   │   ├── settings/
│   │   │   ├── GeneralSettings.tsx
│   │   │   ├── AdvancedSettings.tsx
│   │   │   └── DebugSettings.tsx
│   │   ├── ui/                        -- Composants generiques
│   │   ├── icons/                     -- DictationLogo, MicrophoneIcon
│   │   ├── onboarding/               -- Permissions, ModelCard
│   │   └── model-selector/           -- ModelDropdown, ModelStatusButton
│   └── overlay/
│       └── RecordingOverlay.tsx
├── vendor/
│   └── whisper.cpp/                   -- Sous-module whisper.cpp
├── docs/
│   ├── adr/                           -- Architecture Decision Records
│   ├── specs/                         -- Specifications fonctionnelles
│   └── technical-review-final.md      -- Revue technique 3 sources
├── tests/
│   └── benchmark.rs                   -- Benchmarks latence + zero-edit rate
└── scripts/
    └── build-whisper.sh               -- Compilation whisper.cpp
```

## Annexe C : Theme visuel "cahier de classe"

| Element | Valeur | Usage |
|---------|--------|-------|
| Fond principal | `#faf8f3` (papier creme) | Background de l'app |
| Encre verte | `#2d7a4f` (vert institutionnel) | Texte actif, boutons primaires, overlay |
| Typographie | Serif cursif (titres), sans-serif (corps) | Identite scolaire |
| Son activation | Stylo sur papier (scratch leger) | Feedback auditif debut dictee |
| Son desactivation | Pose de stylo | Feedback auditif fin dictee |
| Overlay | Onde verte pulsante | Indication d'enregistrement actif |
| Icone menu bar | Micro stylise + boucle scripturale | Identite DictAI dans la barre |
