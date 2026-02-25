# Story 1.1 : Demarrer et arreter une dictee par raccourci clavier

Status: done

## Story

En tant qu'utilisateur,
je veux demarrer et arreter l'enregistrement via un raccourci clavier global,
afin de dicter sans quitter l'application en cours.

## Acceptance Criteria

1. **AC1 — Demarrage raccourci global**
   Given l'app DictAI est lancee et le modele Whisper est charge
   When l'utilisateur appuie sur le raccourci global (defaut : Option+Space)
   Then l'enregistrement audio demarre immediatement
   And le raccourci fonctionne dans n'importe quelle application au premier plan

2. **AC2 — Arret par relachement (push-to-talk) ou second appui (toggle)**
   Given un enregistrement est en cours
   When l'utilisateur relache le raccourci (push-to-talk) ou appuie une seconde fois (toggle)
   Then l'enregistrement s'arrete et le pipeline de transcription se declenche

3. **AC3 — Pas de vol de focus**
   Given une autre application a le focus
   When le raccourci global est presse
   Then l'app ne vole pas le focus et l'enregistrement demarre en arriere-plan

## Tasks / Subtasks

- [x] **Task 1 — Configurer le raccourci par defaut** (AC: 1)
  - [x] 1.1 Confirmer que `get_default_settings()` dans `settings.rs` utilise `option+space` pour macOS (conserve du fork Handy — decision utilisateur)
  - [x] 1.2 Mettre a jour la description du binding "transcribe" : "Demarre/arrete la dictee vocale"
  - [x] 1.3 Confirmer que le binding "transcribe_with_post_process" default `option+shift+space` ne conflite pas
  - [x] 1.4 Verifier que le format `option+space` est valide pour les deux backends (Tauri et HandyKeys) — tests unitaires ajoutes

- [x] **Task 2 — Verifier le mode push-to-talk** (AC: 2)
  - [x] 2.1 Verifier que `push_to_talk: true` est le defaut (deja le cas dans `get_default_settings()`)
  - [x] 2.2 Tester : maintenir le raccourci → enregistrement commence, relacher → enregistrement s'arrete
  - [x] 2.3 Verifier que le `TranscriptionCoordinator` gere bien le debounce 30ms sans bloquer

- [x] **Task 3 — Verifier le mode toggle** (AC: 2)
  - [x] 3.1 Quand `push_to_talk: false`, verifier : premier appui → start, deuxieme appui → stop
  - [x] 3.2 Verifier que le coordinator rejette les appuis pendant `Stage::Processing`

- [x] **Task 4 — Valider pas de vol de focus** (AC: 3)
  - [x] 4.1 S'assurer que `init_shortcuts()` utilise un listener global (pas de fenetre requise)
  - [x] 4.2 Verifier que `TranscribeAction::start()` ne fait PAS `app.get_webview_window("main").unwrap().set_focus()`
  - [x] 4.3 Tester avec Chrome, VS Code et Terminal en premier plan — ✅ test manuel passe (utilisateur a dicte depuis une autre app sans vol de focus)

- [x] **Task 5 — Verification pipeline complet start→stop→transcription** (AC: 1, 2)
  - [x] 5.1 Verifier la chaine : raccourci → `handle_shortcut_event()` → `TranscriptionCoordinator::send_input()` → `TranscribeAction::start()` → `AudioRecordingManager::try_start_recording()` → enregistrement
  - [x] 5.2 Verifier la chaine stop : release/toggle → `TranscribeAction::stop()` → `rm.stop_recording()` → `tm.transcribe()` → pipeline → paste
  - [x] 5.3 Verifier que `FinishGuard` remet bien le `TranscriptionCoordinator` a `Stage::Idle`

- [x] **Task 6 — Test d'integration raccourci** (AC: 1, 2, 3)
  - [x] 6.1 Test manuel passe : app lancee, Option+Space presse, parole → texte colle correctement. Deux appuis courts → "Merci. Merci." confirme push-to-talk fonctionnel.
  - [x] 6.2 Modele Whisper charge au demarrage (indicateur vert "Whisper large-v3..." visible en bas de l'UI). Pas de delai observe au premier raccourci.
  - [x] 6.3 Documenter le resultat dans le dev agent record

## Dev Notes

### Etat actuel du code (fork Handy)

**Le raccourci global fonctionne deja.** Le fork Handy fournit une infrastructure shortcut complete :

- `src-tauri/src/shortcut/mod.rs` : Abstraction avec deux backends (`Tauri` global-shortcut plugin + `HandyKeys`). Par defaut macOS → `HandyKeys`.
- `src-tauri/src/shortcut/handler.rs` : Dispatch shared — detecte `is_transcribe_binding()` et redirige vers le `TranscriptionCoordinator`.
- `src-tauri/src/transcription_coordinator.rs` : Machine a etats (`Idle → Recording → Processing → Idle`) avec debounce 30ms. Gere push-to-talk ET toggle.
- `src-tauri/src/actions.rs` : `TranscribeAction` avec `start()` (initiate model load, start recording, register cancel shortcut, audio feedback, tray icon) et `stop()` (stop recording, transcribe, pipeline rules+LLM, paste, hide overlay).
- `src-tauri/src/settings.rs` : `ShortcutBinding` avec `default_binding`, `current_binding`, persistance via tauri-plugin-store.

**Ce qui doit changer :**
1. Le raccourci par defaut est `option+space` (Handy) → le changer en `Cmd+Shift+D` (DictAI spec)
2. Le raccourci post-process est `option+shift+space` → a ajuster si conflit
3. Les descriptions/noms de bindings sont deja en francais (changement anterieur)

### Architecture pertinente

- **Backend** : Rust + Tauri 2.x, macOS only pour MVP
- **Raccourci clavier** : `handy-keys` (default macOS) ou Tauri global-shortcut plugin (fallback)
- **Audio** : `cpal` capture → `rubato` resample 16kHz → `SmoothedVad` (Silero v4) → buffer PCM
- **Pipeline** : orchestrator.rs avec routing conditionnel (confidence >= 0.82 + words <= 30 + mode != Pro → rules-only)
- **Settings** : `tauri-plugin-store` avec `settings_store.json`

### Points d'attention

1. **Le raccourci `cmd+shift+d` peut confliter** avec certains navigateurs (Chrome debug, Firefox). Le systeme de detection de conflits est deja en place (`validate_shortcut_for_implementation()`).
2. **Le modele Whisper doit etre charge** avant le premier raccourci. `TranscribeAction::start()` appelle `tm.initiate_model_load()` — c'est un chargement en arriere-plan. Si le modele n'est pas charge, la transcription echouera. Verifier le comportement.
3. **HandyKeys vs Tauri** : Sur macOS, `HandyKeys` est le defaut. Si ca echoue, fallback auto vers Tauri + persistence dans settings.

### Ce que cette story ne couvre PAS

- Son audio de feedback (Story 2.1)
- Overlay visuel (Story 2.2)
- Modes d'ecriture (Epic 3)
- Collage au curseur (Story 1.4 — separee car logique de paste complexe)
- Configuration du raccourci par l'utilisateur (Story 7.1)

### Project Structure Notes

- Raccourci : `src-tauri/src/shortcut/` (mod.rs, handler.rs, tauri_impl.rs, handy_keys.rs)
- Settings : `src-tauri/src/settings.rs` (struct AppSettings, get_default_settings)
- Actions : `src-tauri/src/actions.rs` (TranscribeAction, ACTION_MAP)
- Coordinator : `src-tauri/src/transcription_coordinator.rs`
- Audio : `src-tauri/src/audio_toolkit/audio/` (recorder.rs, device.rs)
- Pipeline : `src-tauri/src/pipeline/` (orchestrator.rs, rules.rs, modes.rs)

### References

- [Source: src-tauri/src/shortcut/mod.rs] init_shortcuts(), register_shortcut(), change_binding()
- [Source: src-tauri/src/shortcut/handler.rs] handle_shortcut_event() — dispatch push-to-talk/toggle
- [Source: src-tauri/src/transcription_coordinator.rs] TranscriptionCoordinator::new(), Stage enum, debounce 30ms
- [Source: src-tauri/src/actions.rs] TranscribeAction::start/stop, ACTION_MAP
- [Source: src-tauri/src/settings.rs:638-735] get_default_settings() — raccourci defaut, bindings
- [Source: src-tauri/src/pipeline/orchestrator.rs] process() — routing conditionnel
- [Source: {output_folder}/planning-artifacts/architecture.md#ADR-001] Fork Handy, Tauri 2.x
- [Source: {output_folder}/planning-artifacts/epics.md#Story 1.1] Acceptance criteria complets

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (claude-opus-4-6)

### Debug Log References

- `cargo check` : compilation propre (0 erreurs, 2 warnings dead_code preexistants)
- `cargo test` : 100 tests passed (93 unit + 5 benchmark + 2 VAD), 0 failed

### Completion Notes List

- **Task 1** : Raccourci macOS change de `option+space` → `cmd+shift+d`. Post-process change de `option+shift+space` → `cmd+shift+p`. Description binding transcribe mise a jour. Validation confirmee pour les 2 backends (Tauri validation = modifier+key OK, HandyKeys = Hotkey::parse OK). 6 tests unitaires ajoutes dans settings.rs.
- **Task 2** : Code review du TranscriptionCoordinator confirme push-to-talk fonctionne : press→start (Idle), release→stop (Recording). push_to_talk=true est bien le defaut. Debounce 30ms via DEBOUNCE constant + Instant::now() check. 2 tests ajoutes pour is_transcribe_binding().
- **Task 3** : Code review confirme toggle mode : quand push_to_talk=false, premier appui→start (Idle), deuxieme appui→stop (Recording same binding). Le coordinator rejette les appuis pendant Processing (match arm `_ => debug!("Ignoring press")`) et pendant Recording(different binding).
- **Task 4** : init_shortcuts() utilise HotkeyManager global (handy-keys) ou Tauri global-shortcut — aucun ne requiert de fenetre. TranscribeAction::start() NE contient PAS de set_focus(). L'overlay utilise overlay_window.show() sans set_focus. Test manuel requis (Task 6). 6 tests ajoutes dans tauri_impl.rs pour validation shortcut.
- **Task 5** : Pipeline complet verifie par code review : (1) Start chain: handle_shortcut_event → is_transcribe_binding → coordinator.send_input → Command::Input → start() → ACTION_MAP["transcribe"] → TranscribeAction::start() → tm.initiate_model_load() + rm.try_start_recording(). (2) Stop chain: release/toggle → stop() → TranscribeAction::stop() → rm.stop_recording() → tm.transcribe() → pipeline::orchestrator::process() → utils::paste(). (3) FinishGuard (Drop impl) appelle notify_processing_finished() → Stage::Idle.
- **Task 6** : Tests d'integration manuels requis (lancer app, presser Cmd+Shift+D, parler, relacher). Le modele Whisper est charge en arriere-plan via initiate_model_load() au premier raccourci — si pas charge, la transcription echouera mais le systeme ne crashe pas (erreur loguee). 14 tests unitaires ajoutes au total couvrant les changements.

### Change Log

- 2026-02-25 : Story 1.1 implementation — raccourci par defaut change vers Cmd+Shift+D, 14 tests unitaires ajoutes, code review complete pour push-to-talk/toggle/no-focus-steal/pipeline
- 2026-02-25 : Code review adversarial — 2 HIGH, 3 MEDIUM, 2 LOW findings. Fixes appliques : tests HandyKeys ajoutes (H2), tests cross-plateforme (M1), mod tests deplace en fin de fichier (M2), tasks manuelles corrigees (H1).
- 2026-02-25 : Decision utilisateur — garder `option+space` comme raccourci par defaut (plus pratique que `cmd+shift+d`). Revert du changement de raccourci, tests mis a jour. Tests manuels passes par l'utilisateur : transcription OK, push-to-talk OK, pas de vol de focus OK, modele Whisper charge au demarrage.

### File List

- `src-tauri/src/settings.rs` (modifie) — raccourci par defaut, description, tests cross-plateforme
- `src-tauri/src/transcription_coordinator.rs` (modifie) — tests is_transcribe_binding
- `src-tauri/src/shortcut/tauri_impl.rs` (modifie) — tests validation shortcut, mod tests deplace en fin de fichier
- `src-tauri/src/shortcut/handy_keys.rs` (modifie) — tests validation shortcut HandyKeys ajoutes
- `{output_folder}/implementation-artifacts/sprint-status.yaml` (modifie) — status tracking
- `{output_folder}/implementation-artifacts/1-1-demarrer-arreter-dictee-raccourci-clavier.md` (modifie) — story file
