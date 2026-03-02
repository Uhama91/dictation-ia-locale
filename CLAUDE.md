# CLAUDE.md — Dictation IA Locale (DictAI)

Application de dictée vocale **local-first** inspirée de Wispr Flow.
Pipeline : raccourci clavier → capture audio → Whisper (STT) → LLM (post-traitement) → collage automatique au curseur.

---

## Contexte projet

- **Statut** : Implémentation terminée — Epics 1-7 done, backlog dark mode
- **Cible** : macOS (menu bar app)
- **Philosophie** : Privacy-first, local par défaut, cloud opt-in
- **Repo** : `https://github.com/Uhama91/dictation-ia-locale`
- **Fork de** : Handy (branding migré vers DictAI)

---

## Stack technique

| Composant | Technologie |
|-----------|-------------|
| Frontend | React 18 + TypeScript + Tailwind CSS 4 |
| Backend | Tauri v2 (Rust) |
| STT | Whisper.cpp (via whisper-rs FFI) |
| LLM local | Ollama (HTTP, fallback rules-only) |
| Stockage | SQLite (via tauri-plugin-sql) |
| UI | Menu bar macOS + fenêtre settings |
| Tests | Vitest + React Testing Library (53 tests) |
| i18n | react-i18next (FR/EN) |
| State | Zustand 5 |

---

## Architecture pipeline

```
Raccourci clavier → VAD → Capture audio (micro) → Whisper (STT) → Rules cleanup → LLM (si nécessaire) → Collage curseur
```

## Modes d'écriture

- **Chat** : correction orthographe minimale, ponctuation basique, ton conservé
- **Pro** : reformulation concise, style email, paragraphes clairs
- **Code** : jargon technique préservé, formatage Markdown, symboles intacts

---

## Thème visuel — "Cahier de classe" (Epic 6.1)

| Token | Valeur |
|-------|--------|
| Fond papier crème | `#faf8f3` |
| Encre verte (primaire) | `#2d7a4f` |
| Encre bleue (secondaire) | `#1a56db` |
| Rouge correction (erreur) | `#b91c1c` |
| Texte encre sombre | `#1a1a2e` |
| Police branding | Caveat |
| Police UI | Inter |
| Police code | JetBrains Mono |

---

## Commandes utiles

```bash
# Frontend
npm run dev          # Dev server React (sans Tauri)
npm test             # Vitest — 53 tests
npm run test:watch   # Vitest mode watch
npm run test:coverage

# Tauri
npm run tauri dev    # App complète (Rust + React)
cargo test --lib     # Tests Rust uniquement
```

---

## Tests frontend

- **Framework** : Vitest 4 + React Testing Library + jsdom
- **Config** : `vitest.config.ts`
- **Setup global** : `src/test/setup.ts` (14 mocks Tauri + bindings + clipboard + matchMedia)
- **Helper** : `src/test/helpers.tsx` → `renderWithI18n()` avec i18n isolé
- **Pattern Tauri events** : `mockListenWithCapture()` + `act()` + `waitFor()`
- **Pattern Zustand** : `store.setState()` direct dans les tests

---

## Avancement Epics

| Epic | Statut |
|------|--------|
| 1 — Dictée vocale fondamentale | Done |
| 2 — Retour visuel et sonore | Done |
| 3 — Modes d'écriture intelligents | Done |
| 4 — Historique des dictées | Done |
| 5 — Onboarding, modèles et vie privée | Done |
| 6 — Identité visuelle DictAI | Done |
| 7 — Paramètres et personnalisation | Done |
| Backlog — Dark mode toggle | Noté dans epics.md |

---

## Story 6.2 — Navigation simplifiée (DONE)

Sidebar refactorée : 7 sections → 3 sections (**Accueil**, **Style**, **Paramètres**).
- **AccueilSettings** : indicateur état dictée temps réel, raccourci, sélecteur mode, aperçu dernière dictée
- **StyleSettings** : 3 cartes mode d'écriture avec exemples avant/après
- **ParametresSettings** : accordéons (Audio, Raccourcis, Modèles, Historique, Avancé, Post-traitement, À propos, Debug)
- Code review Gemini passée : 4 findings corrigés (memory leak listeners, disabled state, dead code, sémantique button)

---

## Workflow EPCT

**Pour toute tâche** : Explore → Plan → Code → Test → Write Up

- Privilégier `Edit` sur `Write`
- Sécuriser les credentials (pas de clés en dur)
- Cross-LLM review avec Gemini à chaque étape

---

## Session Log

| Date | Action | Fichiers |
|------|--------|----------|
| 2026-02-24 | Initialisation + BMAD Quick Spec + Tasks 1-22 implémentées | src-tauri/, src/ |
| 2026-02-27 | Epic 5 complète + retro + sprint stabilisation tests (53 tests) | test/setup.ts, 7 fichiers .test.tsx |
| 2026-02-28 | Story 6.1 : thème cahier de classe, polices, suppression legacy Handy | App.css, tailwind.config.js, DictationLogo.tsx, main.tsx |
| 2026-02-28 | Story 6.2 : exploration terminée, plan de navigation 3 sections prêt | Plan dans .claude/plans/ |
| 2026-03-01 | Story 6.2 : implémentation + code review Gemini (4 fixes) | Sidebar.tsx, AccueilSettings, StyleSettings, ParametresSettings |
| 2026-03-02 | Story 6.3 : page À propos — identité DictAI, licence MIT, crédits enrichis, UpdateChecker | AboutSettings.tsx, translation.json (FR/EN) |
| 2026-03-02 | Story 7.1 : raccourci single-key (Option/⌘) — PTT 200ms + double-tap mains libres | single_key.rs, mod.rs, settings.rs, TriggerKeySelector.tsx, bindings.ts |
| 2026-03-02 | Epic 7 done — 7.2/7.3/7.4 déjà couverts par héritage Handy. Toutes epics terminées. | sprint-status.yaml, CLAUDE.md |
