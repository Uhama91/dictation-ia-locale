# Revue Technique Comparative — Dictation IA Locale

**Date :** 2026-02-24
**Sources :** Recherche web approfondie (20+ sources, arXiv, GitHub, benchmarks 2025-2026)

---

## Verdict global

La tech spec actuelle (Rust + Tauri, Whisper GGML, LLM < 2B, Silero VAD) est **solide mais sous-optimale** sur plusieurs axes. Des alternatives concrètes existent sur chaque composant, avec des gains de latence significatifs accessibles.

---

## 1. Risques techniques majeurs

### 🔴 Risque #1 — Fine-tuning LLM (risque le plus élevé)
- Construire 5-10k paires FR de qualité demande énormément de travail manuel ou une génération synthétique à valider
- Le zero-edit rate > 70% **dépend entièrement de la qualité du dataset** — c'est le point le plus incertain
- **Mitigation** : utiliser Common Voice FR + MLS FR + génération synthétique (voir section 5)

### 🔴 Risque #2 — Latence cible ambitieuse
- < 2s avec deux modèles d'inférence séquentiels (Whisper → LLM) sur machines 6-8 Go RAM est **très ambitieux**
- Wispr Flow atteint < 700ms avec TensorRT-LLM sur GPU cloud — **incomparable avec llama.cpp local**
- **Mitigation** : WhisperKit CoreML + Qwen2.5-0.5B Q4 peuvent atteindre 650-850ms sur M2/M3

### 🟡 Risque #3 — Dépendance à transcribe-rs
- `transcribe-rs` est une crate non officielle qui unifie plusieurs moteurs STT mais ajoute une couche d'abstraction
- Moins de contrôle sur les optimisations CoreML/Metal spécifiques à macOS
- **Mitigation** : utiliser whisper.cpp directement via FFI Rust pour accès aux optimisations CoreML encoder

### 🟡 Risque #4 — Menu bar macOS via Tauri
- Tauri offre un contrôle minimal sur le comportement natif de la menu bar macOS
- Les interactions se sentent "datées" comparées aux apps natives (retour terrain 2025)
- **Mitigation** : plugin Swift pour la menu bar (voir section 4)

---

## 2. STT — Meilleures alternatives pour le français

### Recommandation principale : Whisper large-v3-turbo Q5_0

| Modèle | Latence (Apple Silicon) | RAM | Précision FR |
|--------|------------------------|-----|--------------|
| Whisper small Q5 (actuel) | ~400ms | ~460 Mo | Bonne |
| Whisper medium Q4 (actuel) | ~600ms | ~492 Mo | Très bonne |
| **Whisper large-v3-turbo Q5_0** | **~600-800ms** | **~800 Mo** | **Excellente** |
| **WhisperKit CoreML (ANE)** | **~450ms** | **~800 Mo** | **Excellente** |

**Whisper large-v3-turbo** = décodeur réduit de 32 → 4 couches (5.4x plus rapide que large-v3), précision quasi identique au français. WER français : 3-8% sur données propres.

**Optimisation cruciale** : utiliser whisper.cpp compilé avec `WHISPER_COREML=1` :
- Encoder sur Apple Neural Engine (ANE) : **3x plus rapide**
- Décodeur sur Metal GPU : **3-4x plus rapide**
- Combinaison CoreML encoder + Metal decoder → latence ~450-600ms vs ~1.5s sans

```bash
# Compilation whisper.cpp avec CoreML
cmake -B build -DWHISPER_COREML=1 -DWHISPER_METAL=1
```

### Alternatives à considérer

- **WhisperKit (Swift/CoreML)** : latence 0.45s sur M3 Max, mais nécessite Swift (voir section 4)
- **Moonshine** : ne supporte pas le français — à éliminer
- **Parakeet NVIDIA** : anglais uniquement pour les performances optimales — à éliminer
- **MMS / Wav2Vec2 FR** : dépassés par Whisper large-v3 — à éliminer

---

## 3. LLM de nettoyage — Taille optimale

### Un LLM < 1B params peut suffire ✅

**Recommandation : Qwen2.5-0.5B-Instruct Q4**
- Taille : ~300 Mo RAM (vs ~1.2 Go pour 1.5B)
- Supporte officiellement le français (29 langues)
- Suffisant pour une tâche contrainte (correction, non génération libre)
- Fine-tuning QLoRA supporté officiellement

**Comparatif modèles < 2B :**

| Modèle | RAM (Q4) | FR natif | Recommandation |
|--------|----------|----------|----------------|
| **Qwen2.5-0.5B Q4** | ~300 Mo | ✅ Oui | ⭐ Premier choix |
| Qwen2.5-1.5B Q4 | ~900 Mo | ✅ Oui | Fallback si 0.5B insuffisant |
| SmolLM2-135M | ~100 Mo | ❌ Faible | Insuffisant seul pour FR |
| SmolLM2-360M | ~230 Mo | ❌ Faible | Insuffisant seul pour FR |
| Phi-3.5-mini (3.8B) | ~2.2 Go | ✅ Bien | Trop lourd pour le budget |

### Approche hybride recommandée (meilleur ratio qualité/latence)

```
Transcription brute
    ↓
[Règles rapides < 1ms]
  - Filler words FR (euh, heu, bah, ben, du coup, genre, voilà)
  - Bégaiements
  - Ponctuation basique (majuscule début, point final)
    ↓
Score de confiance Whisper élevé ? → Coller directement (no LLM)
Score bas ou phrases longues ? → Qwen2.5-0.5B Q4 (nettoyage complet)
```

**Résultat attendu** : 60-65% des cas traités sans LLM (< 5ms), LLM uniquement pour les 35-40% restants (~150-200ms).

---

## 4. Architecture — Swift vs Rust + Tauri

### Recommandation : Architecture hybride Rust + plugin Swift

Ne pas abandonner Handy/Rust, mais ajouter une couche Swift pour les composants macOS critiques.

```
┌──────────────────────────────────────────────────┐
│  Tauri 2.x (Web frontend — UI settings légère)  │
├──────────────────────────────────────────────────┤
│  Rust backend (orchestration + LLM)              │
│  ├── VAD (Silero via ONNX / vad-rs)              │
│  ├── LLM cleanup (Qwen2.5-0.5B via llama-cpp-rs) │
│  └── Pipeline orchestration (async channels)    │
├──────────────────────────────────────────────────┤
│  Plugin Swift (FFI depuis Rust) — composants clés│
│  ├── WhisperKit (CoreML + ANE) — STT ultra-rapide│
│  ├── Accessibility API (paste au curseur natif)  │
│  ├── Global hotkeys (CGEvent/Carbon)             │
│  └── Menu bar native (NSStatusBar)               │
└──────────────────────────────────────────────────┘
```

**Pourquoi pas Swift pur ?**
- Abandonner Handy = tout réécrire
- Rust reste meilleur pour le pipeline bas niveau (VAD, buffers audio, async)
- llama-cpp-rs est mature pour Rust

**Pourquoi ajouter Swift ?**
- WhisperKit ANE : 3-5x plus rapide que GGML Metal seul
- Menu bar macOS native = expérience utilisateur correcte
- Accessibility API sans bridge complexe

---

## 5. Optimisation de la latence

### Projection avec stack optimisée

```
Raccourci pressé
    ↓
Capture audio (M1/M2/M3 — AVFoundation ou cpal)
    ↓
TEN VAD (latence réduite vs Silero) détecte fin de parole    ~50ms
    ↓
Whisper large-v3-turbo Q5_0
  - Encoder CoreML/ANE                                       ~150ms
  - Décodeur Metal                                           ~300ms
  Total STT                                                  ~450-600ms
    ↓
Qwen2.5-0.5B Q4 (si nécessaire)                             ~150-200ms
    ↓
Paste au curseur (Accessibility API natif)                   ~10ms

TOTAL ESTIMÉ (M2/M3)                                        ~650-860ms
TOTAL ESTIMÉ (M1)                                           ~1.0-1.4s
```

### VAD — Upgrade recommandé : TEN VAD

**TEN VAD** (GitHub: TEN-framework) présente une latence switch speech→silence significativement réduite par rapport à Silero v4. Benchmark 2025 (Picovoice) : TEN VAD est dans le top 3 des VAD pour la détection de fin de parole. Gain : -100 à -200ms sur la détection de fin d'enregistrement.

### Pipeline asynchrone STT + LLM (Phase 2)

Gain théorique de 200-400ms en faisant commencer le LLM sur les premiers tokens STT :

```
[STT démarre] → [tokens partiels à 0.5s] → [LLM commence sur partiel]
               → [STT finit à 0.8s] → [LLM complète le delta]
Total avec overlap : ~700-900ms (vs ~1.1s séquentiel)
```

À implémenter en Phase 2 après validation du pipeline de base.

---

## 6. Dataset fine-tuning FR

### Sources de données disponibles

| Source | Volume | Type | Qualité |
|--------|--------|------|---------|
| **Mozilla Common Voice FR v18+** | 1 200h validées | Lectures + corrections humaines | ⭐⭐⭐ |
| **MLS (Multilingual LibriSpeech) FR** | ~1 100h | Livres audio + textes alignés | ⭐⭐⭐ |
| **ESLO / TCOF (ORTOLANG/CNRS)** | ~200h | Français parlé spontané | ⭐⭐⭐⭐ |
| **Génération synthétique LLM** | Illimitée | Simulée | ⭐⭐ |

### Architecture dataset recommandée (10k paires)

```
10k paires total :
├── 3k : Common Voice FR → Whisper small (brut) / transcription humaine (propre)
├── 3k : MLS FR (même méthode)
├── 2k : ESLO/TCOF (français spontané, le plus difficile)
└── 2k : Synthétique LLM (augmentation, cas difficiles : homophones FR, liaisons)

Homophones FR à couvrir : ces/ses/c'est, à/a, ou/où, et/est, on/ont, son/sont
```

### Méthode génération synthétique

1. Partir de texte FR propre (Wikipedia, news)
2. Simuler erreurs STT via LLM : supprimer ponctuation, ajouter erreurs homophoniques FR
3. Qualité attendue : 70-80% des données réelles pour cette tâche contrainte

---

## Synthèse des recommandations prioritaires

| Composant | Actuel | Recommandé | Impact |
|-----------|--------|-----------|--------|
| **STT modèle** | Whisper small/medium Q4 | Whisper large-v3-turbo Q5_0 | +15-20% précision FR |
| **STT runtime** | transcribe-rs GGML | whisper.cpp CoreML encoder + Metal | 3-5x vitesse encoder |
| **VAD** | Silero v4 | TEN VAD | -100-200ms fin de phrase |
| **LLM cleanup** | Qwen2.5-1.5B ou SmolLM2-1.7B | **Qwen2.5-0.5B Q4 + règles hybrides** | -50% RAM, latence similaire |
| **Framework** | Rust + Tauri seul | Rust + Tauri + **plugin Swift** | Accès ANE + menu bar native |
| **Pipeline** | Séquentiel | Async STT→LLM overlap (Ph. 2) | -200-400ms latence |

---

## Sources

- [mac-whisper-speedtest benchmarks](https://github.com/anvanvan/mac-whisper-speedtest)
- [WhisperKit On-device Real-time ASR — arXiv 2507.10860](https://arxiv.org/html/2507.10860v1)
- [Whisper large-v3-turbo — Whisper Notes Blog](https://whispernotes.app/blog/introducing-whisper-large-v3-turbo)
- [Best open source STT 2026 — Northflank benchmarks](https://northflank.com/blog/best-open-source-speech-to-text-stt-model-in-2026-benchmarks)
- [Parakeet v3 NVIDIA — Caasify](https://caasify.com/blog/parakeet-v3-nvidias-asr-model-competing-with-whisper/)
- [TEN VAD low-latency — GitHub](https://github.com/TEN-framework/ten-vad)
- [Best VAD 2025 — Picovoice](https://picovoice.ai/blog/best-voice-activity-detection-vad-2025/)
- [whisper.cpp CoreML 3x speedup — HN](https://news.ycombinator.com/item?id=43880345)
- [WhisperKit — GitHub argmaxinc](https://github.com/argmaxinc/WhisperKit)
- [Tauri macOS menu bar limites](https://github.com/tauri-apps/tauri/discussions/6223)
- [candle-coreml Rust ANE bindings](https://crates.io/crates/candle-coreml)
- [Bloomberg Streaming Whisper Interspeech 2025](https://www.bloomberg.com/company/stories/bloombergs-ai-researchers-turn-whisper-into-a-true-streaming-asr-model-at-interspeech-2025/)
- [Concurrent voice AI pipelines — Gladia](https://www.gladia.io/blog/concurrent-pipelines-for-voice-ai)
- [SmolLM2 — arXiv 2502.02737](https://arxiv.org/html/2502.02737v1)
- [Qwen2.5-1.5B-Instruct — Hugging Face](https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct)
