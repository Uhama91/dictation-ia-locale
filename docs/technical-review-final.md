# Synthèse Technique Finale — Dictation IA Locale

**Date :** 2026-02-24
**Sources croisées :** Recherche web (20+ articles/benchmarks 2025-2026) + Gemini 3 Pro + GPT-5.2

---

## Verdict global

**3 modèles convergent** sur les mêmes conclusions majeures. L'architecture est solide, mais sous-optimale sur 4 axes. Voici les recommandations consolidées.

---

## Convergences (unanimes entre les 3 sources)

| Point | Consensus |
|-------|-----------|
| STT modèle | Whisper `large-v3-turbo` Q4/Q5 >> small/medium |
| LLM cleanup | Qwen2.5-0.5B suffit pour la tâche (< 1B params OK) |
| Framework long terme | Swift natif + CoreML = architecture optimale macOS |
| Framework MVP | Rust + Tauri OK pour livrer vite (fork Handy = gain de temps) |
| Latence | Pipeline hybride règles + LLM conditionnel est la clé |

---

## 1. STT — Recommandation consolidée

### ✅ Changer : Whisper small/medium → large-v3-turbo Q5_0

**Pourquoi :** Le large-v3-turbo a le décodeur réduit de 32 → 4 couches, soit 5.4x plus rapide que large-v3 avec une précision quasi identique en français.
- WER français : **3-8%** (vs 12-15% pour small)
- Ponctuation et majuscules gérées nativement → réduit le travail du LLM
- Taille : ~800 Mo (budget RAM OK)

**Optimisation clé (unanime) :** Compiler whisper.cpp avec `WHISPER_COREML=1` + `WHISPER_METAL=1` :
```bash
cmake -B build -DWHISPER_COREML=1 -DWHISPER_METAL=1
```
→ Encoder sur ANE (Apple Neural Engine) : **3x plus rapide**
→ Décodeur sur Metal GPU : **3-4x plus rapide**
→ Latence STT totale : **450-600ms** au lieu de ~1.5s sans optimisation

**Params Whisper à optimiser (GPT-5.2) :**
```rust
WhisperConfig {
    language: "fr",
    beam_size: 1,        // Défaut 5, réduit pour latence (-100ms)
    best_of: 1,          // Défaut 5
    temperature: 0.0,    // Greedy = plus rapide
    no_speech_threshold: 0.6,
}
```

**Alternative GPT : Faster-Whisper (CTranslate2)** — 2-3x plus rapide que GGML sur CPU. À benchmarker si l'intégration Rust est disponible (`faster-whisper-rs`). Potentiellement < 300ms.

---

## 2. LLM de nettoyage — Recommandation consolidée

### ✅ Changer : Qwen2.5-1.5B → Qwen2.5-0.5B Q4

**Unanime entre les 3 sources.** La tâche est étroite (correction, non génération créative) → un modèle 0.5B fine-tuné peut atteindre > 70% zero-edit rate.

| Modèle | RAM Q4 | Latence ~50 tok | FR natif | Verdict |
|--------|--------|-----------------|----------|---------|
| **Qwen2.5-0.5B Q4** | ~300 Mo | **200-300ms** | ✅ | ⭐ Premier choix |
| Qwen2.5-1.5B Q4 | ~900 Mo | 500-700ms | ✅ | Fallback si 0.5B insuffisant |
| SmolLM2-360M | ~230 Mo | 150-250ms | ❌ FR faible | Risque hallucinations FR |
| Llama-3.2-1B | ~700 Mo | 350-500ms | ✅ | Alternative si Qwen OOS |

**Params LLM à optimiser (GPT-5.2) :**
```rust
LlamaParams {
    n_predict: 128,       // Limite tokens générés
    temperature: 0.0,     // Greedy sampling = plus rapide
    top_k: 1,
    repeat_penalty: 1.0,
}
```

### ✅ Approche hybride règles + LLM conditionnel (Gemini + Research Agent)

Ne pas appeler le LLM systématiquement :

```
Transcription brute Whisper
    ↓
[Règles rapides < 1ms]
  → Filler words FR : "euh", "heu", "bah", "ben", "du coup", "genre", "voilà"
  → Bégaiements collapse
  → Ponctuation basique (majuscule début, point final)
    ↓
Score confiance Whisper élevé + phrases courtes ? → Coller directement (sans LLM)
Score bas / phrases longues / mode Pro/Code ? → Qwen2.5-0.5B (nettoyage complet)
```

**Résultat attendu :** 60-65% des cas résolus sans LLM (< 5ms). LLM uniquement pour 35-40% → latence moyenne réduite.

### ✅ Insight Gemini — Tester sans LLM d'abord

> *"Whisper large-v3-turbo est si bon en français qu'il gère nativement la ponctuation, les majuscules et ignore souvent les disfluences. Ton zero-edit rate pourrait dépasser 70% sans LLM du tout."*

**Action recommandée :** Faire un benchmark de zero-edit rate avec Whisper large-v3-turbo SEUL avant d'implémenter le LLM. Si > 70% → le LLM devient optionnel.

---

## 3. Architecture — Recommandation consolidée

### MVP : Rust + Tauri (maintenir le fork Handy)

Les 3 sources sont d'accord : Rust/Tauri est pragmatique pour livrer le MVP. Le fork Handy économise des semaines de développement.

**Amélioration immédiate (Research Agent) :** Architecture hybride avec plugin Swift pour les composants critiques :

```
┌──────────────────────────────────────────────────┐
│  Tauri 2.x (frontend UI — léger, settings)      │
├──────────────────────────────────────────────────┤
│  Rust backend (orchestration + LLM)              │
│  ├── VAD (TEN VAD ou Silero via vad-rs)          │
│  ├── LLM cleanup (Qwen2.5-0.5B via llama-cpp-rs) │
│  └── Pipeline async (channels Tokio)             │
├──────────────────────────────────────────────────┤
│  Plugin Swift (FFI depuis Rust) — perf critique  │
│  ├── Whisper + CoreML encoder ANE                │
│  ├── Accessibility API (paste natif au curseur)  │
│  └── Menu bar NSStatusBar natif                  │
└──────────────────────────────────────────────────┘
```

### Post-MVP V2 : Migration vers Swift natif + CoreML

Les 3 sources recommandent ce pivot pour la V2 :

| Aspect | Rust + Tauri | Swift natif + CoreML |
|--------|-------------|---------------------|
| RAM au repos | ~200 Mo (WebView) | **~30 Mo** |
| Latence LLM | llama.cpp Metal | **CoreML ANE (2-3x plus rapide)** |
| Menu bar macOS | Artificiel | **Natif NSStatusBar** |
| Accessibility paste | Bridge FFI | **API directe AXUIElement** |
| Audio capture | cpal | **AVFoundation** |
| Distribution | Complexe | **Notarisation + App Store simple** |

---

## 4. VAD — Upgrade recommandé

### Passer de Silero v4 → TEN VAD

**TEN VAD** (GitHub: TEN-framework/ten-vad) a une latence switch speech→silence réduite. Top 3 des VAD 2025 (benchmark Picovoice). Gain : -100 à -200ms sur la détection de fin de parole.

---

## 5. Dataset fine-tuning FR

### Sources disponibles (Research Agent)

| Source | Volume | Qualité |
|--------|--------|---------|
| Mozilla Common Voice FR v18+ | 1 200h validées | ⭐⭐⭐ |
| MLS (Multilingual LibriSpeech) FR | ~1 100h | ⭐⭐⭐ |
| ESLO/TCOF (ORTOLANG/CNRS) | ~200h français spontané | ⭐⭐⭐⭐ |
| Génération synthétique LLM | Illimitée | ⭐⭐ |

**Architecture 10k paires recommandée :**
```
10k paires :
├── 3k : Common Voice FR → Whisper small (brut) / humain (propre)
├── 3k : MLS FR (même méthode)
├── 2k : ESLO/TCOF (français spontané — représente mieux la dictée)
└── 2k : Synthétique LLM (homophones FR : ces/ses/c'est, à/a, ou/où)
```

### Workflow GPT-5.2 : Baseline avant fine-tuning

1. **Baseline zéro-shot** avec Qwen2.5-0.5B sans fine-tuning → mesure le zero-edit rate natif
2. Si baseline > 50% → fine-tuning SFT LoRA sur 5-10k paires
3. Si baseline > 65% avec Whisper turbo seul → reconsidérer si le LLM est nécessaire

---

## 6. Fonctionnalités architecturales uniquement chez GPT-5.2

### Mode Rapide / Mode Qualité (excellente suggestion UX)

```
Mode Rapide (raccourci 1) — cible < 500ms
  → Whisper large-v3-turbo + règles regex
  → Pas de LLM
  → Pour : Slack, chats rapides

Mode Qualité (raccourci 2) — cible < 1.5s
  → Whisper + Qwen2.5-0.5B nettoyage complet
  → Modes Chat/Pro/Code
  → Pour : emails, documents, code
```

### Feedback utilisateur → amélioration continue

- Bouton "Corriger" dans l'historique
- La correction manuelle génère une paire (brut → corrigé) sauvegardée localement
- Export périodique pour ré-entraîner le modèle sur les vraies données de l'utilisateur

### Benchmark automatisé continu

```rust
// tests/benchmark.rs — À lancer à chaque commit
fn benchmark_pipeline() {
    let audio_fixtures = load_fr_audio_corpus(); // 100 phrases FR variées
    // Mesure latence p50/p99 + zero-edit rate
}
```

---

## Projection de latence finale (stack optimisée)

```
Raccourci pressé
    ↓
Capture audio (cpal / AVFoundation)
    ↓
TEN VAD détecte fin de parole                   ~50ms
    ↓
Whisper large-v3-turbo Q5_0
  - Encoder CoreML/ANE                          ~150ms
  - Décodeur Metal                              ~300ms
  Total STT                                     ~450-600ms
    ↓
Règles locales (toujours)                       < 1ms
Score confiance élevé ? → Coller direct
Sinon → Qwen2.5-0.5B Q4                        ~200-300ms
    ↓
Paste au curseur (Accessibility API)            ~10ms

TOTAL (M2/M3, cas avec LLM)                    ~700-950ms ✅
TOTAL (M2/M3, cas sans LLM)                    ~500-650ms ✅
TOTAL (M1, cas avec LLM)                       ~1.0-1.4s  ✅
```

**→ Objectif < 1.5s atteint sur tous les Mac M-series avec la stack optimisée.**

---

## Plan d'action prioritaire mis à jour

### Critiques — À intégrer dans la tech spec

1. **STT :** Whisper large-v3-turbo Q5_0 (pas small/medium)
2. **STT runtime :** whisper.cpp avec `WHISPER_COREML=1 WHISPER_METAL=1` (pas transcribe-rs)
3. **LLM :** Qwen2.5-0.5B Q4 (pas 1.5B) comme premier candidat
4. **Architecture :** Ajouter plugin Swift pour ANE + menu bar + Accessibility
5. **VAD :** Évaluer TEN VAD vs Silero (benchmark)
6. **Workflow :** Baseline sans LLM d'abord (Whisper turbo seul), mesurer zero-edit rate

### Importantes — Post-MVP V1

7. Mode Rapide / Mode Qualité avec raccourcis distincts
8. Pipeline async STT→LLM overlap (-200-400ms latence)
9. Feedback utilisateur → paires pour ré-entraînement

### V2

10. Migration full Swift natif + CoreML (ANE pour LLM, WhisperKit)
11. Streaming STT (chunking pendant l'enregistrement)
