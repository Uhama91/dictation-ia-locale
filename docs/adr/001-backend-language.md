# Architecture Decision Records

## ADR 001: Choix du langage backend

**Statut** : Proposition  
**Date** : 2026-02-23

### Contexte
Besoin d'un langage pour le backend de l'application de dictée.

### Options considérées
1. **Python** : Écosystème riche (Whisper, ML), itération rapide
2. **Swift** : Intégration macOS native, meilleures perfs, distribution plus simple

### Décision
À déterminer lors de la Phase 0.

### Conséquences
- Python = plus rapide à prototyper
- Swift = meilleure expérience utilisateur final sur macOS

---

## ADR 002: Modèle STT

**Statut** : Proposition  
**Date** : 2026-02-23

### Contexte
Besoin d'un modèle de Speech-to-Text local.

### Options considérées
1. **Whisper (OpenAI)** : Référence, multilingue, précis
2. **Whisper.cpp** : Version C++ optimisée, plus rapide
3. **Faster-Whisper** : Optimisé pour GPU/CPU

### Décision préliminaire
Faster-Whisper ou Whisper.cpp selon les benchmarks sur le hardware cible.

---

## ADR 003: LLM local

**Statut** : Proposition  
**Date** : 2026-02-23

### Contexte
Besoin d'un LLM pour le post-traitement du texte.

### Options considérées
1. **Ollama** : Simple à installer, gestion des modèles facile
2. **llama.cpp** : Plus léger, plus complexe
3. **API cloud** (OpenAI, Anthropic) : Optionnel, opt-in

### Décision préliminaire
Ollama pour le local, avec fallback cloud optionnel.
