# CLAUDE.md — Dictation IA Locale

Application de dictée vocale **local-first** inspirée de Wispr Flow.
Pipeline : raccourci clavier → capture audio → Whisper (STT) → LLM (post-traitement) → collage automatique au curseur.

---

## Contexte projet

- **Statut** : Phase 0 — cadrage, zéro code source
- **Cible** : macOS (menu bar app)
- **Philosophie** : Privacy-first, local par défaut, cloud opt-in
- **Repo** : `https://github.com/Uhama91/dictation-ia-locale`

---

## Stack technique (en cours de décision)

| Composant | Options | Décision |
|-----------|---------|----------|
| Backend | Python / Swift | À valider (Phase 0) |
| STT | Faster-Whisper / Whisper.cpp | Faster-Whisper préféré |
| LLM local | Ollama | Validé (+ fallback cloud opt-in) |
| Stockage | SQLite | Validé |
| UI | Menu bar macOS | Validé |

---

## Architecture pipeline

```
Raccourci clavier → Capture audio (micro) → STT (Whisper) → LLM (nettoyage) → Collage curseur
```

## Modes d'écriture

- **Chat** : correction orthographe minimale, ponctuation basique, ton conservé
- **Pro** : reformulation concise, style email, paragraphes clairs
- **Code** : jargon technique préservé, formatage Markdown, symboles intacts

---

## Structure source (à créer)

```
src/
├── audio/    # Capture micro
├── stt/      # Speech-to-text (Whisper)
├── llm/      # Post-traitement LLM
├── input/    # Raccourcis clavier globaux
└── ui/       # Menu bar interface
```

---

## Workflow EPCT

**Pour toute tâche** : Explore → Plan → Code → Test → Write Up

- Privilégier `Edit` sur `Write`
- Sécuriser les credentials (pas de clés en dur)
- Tests : `pytest tests/`

---

## Règles Claude Code

- Phase 0 active : valider les décisions d'archi avant de coder
- Chaque ADR modifié → mettre à jour `docs/adr/`
- Chaque session avec modifications → mettre à jour Session Log ci-dessous

---

## Session Log

| Date | Action | Fichiers |
|------|--------|----------|
| 2026-02-24 | Initialisation CLAUDE.md + installation BMAD Method | CLAUDE.md, .bmad-core/ |
| 2026-02-24 | BMAD Quick Spec Steps 1-4 terminés — tech-spec ready-for-dev. Revue 3 sources intégrée. | tech-spec-dictation-ia-locale-mvp.md, docs/technical-review-final.md |
