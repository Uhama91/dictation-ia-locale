---
stepsCompleted: ['step-01-validate-prerequisites', 'step-02-design-epics', 'step-03-create-stories', 'step-04-final-validation']
status: 'complete'
completedAt: '2026-02-25'
inputDocuments:
  - '{output_folder}/planning-artifacts/prd.md'
  - '{output_folder}/planning-artifacts/architecture.md'
  - '{output_folder}/planning-artifacts/ux-design.md'
---

# DictAI - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for DictAI, decomposing the requirements from the PRD, UX Design if it exists, and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

- FR1: L'utilisateur peut démarrer une dictée via un raccourci clavier global configurable, depuis n'importe quelle application
- FR2: L'utilisateur peut arrêter la dictée en relâchant le raccourci ou via un second appui
- FR3: Le système peut capturer l'audio du microphone et le transcrire en texte français via Whisper
- FR4: Le système peut détecter automatiquement le début et la fin de parole (VAD)
- FR5: Le système peut appliquer des règles de nettoyage FR au texte transcrit (filler words, élisions, ponctuation doublée, bégaiements, espacement)
- FR6: Le système peut router conditionnellement le texte vers le LLM local selon le score de confiance, le nombre de mots et le mode d'écriture
- FR7: Le système peut coller automatiquement le texte traité au curseur actif dans l'application au premier plan
- FR8: L'utilisateur peut sélectionner un mode d'écriture parmi Chat, Pro et Code
- FR9: En mode Chat, le système peut appliquer une correction orthographique minimale et une ponctuation basique tout en conservant le ton original
- FR10: En mode Pro, le système peut reformuler le texte de manière concise et professionnelle via le LLM local
- FR11: En mode Code, le système peut préserver le jargon technique et formater en Markdown
- FR12: Le système peut jouer un son de confirmation à l'activation de la dictée
- FR13: Le système peut afficher un overlay visuel pendant la dictée indiquant que l'enregistrement est actif
- FR14: L'utilisateur peut voir l'état de la dictée (inactive, écoute, traitement) via l'overlay
- FR15: Le système peut enregistrer chaque session de dictée dans un historique local (texte brut, texte traité, horodatage, mode)
- FR16: L'utilisateur peut consulter l'historique des dictées passées
- FR17: L'utilisateur peut copier un texte depuis l'historique
- FR18: L'utilisateur peut configurer le raccourci clavier global
- FR19: L'utilisateur peut choisir le microphone source
- FR20: L'utilisateur peut activer/désactiver le post-traitement LLM
- FR21: L'utilisateur peut activer/désactiver le mode debug
- FR22: L'utilisateur peut activer le lancement automatique au démarrage macOS
- FR23: L'utilisateur peut accéder aux paramètres via l'icône menu bar
- FR24: L'utilisateur peut naviguer entre les sections de l'interface (Accueil, Style, Paramètres)
- FR25: L'utilisateur peut voir les informations de l'application (version, licence, crédits)
- FR26: L'interface peut afficher le thème "cahier de classe" (palette papier crème / encre verte)
- FR27: Le système peut fonctionner entièrement hors ligne sans aucune connexion réseau
- FR28: Le système peut demander les permissions macOS nécessaires (Microphone, Accessibility) avec des explications en français
- FR29: L'utilisateur peut vérifier qu'aucune donnée ne quitte sa machine (indicateur de statut réseau)
- FR30: En mode debug, l'utilisateur peut voir les métriques du pipeline (temps VAD, temps Whisper, confiance, route, temps rules/LLM, temps total)
- FR31: En mode debug, l'utilisateur peut voir le texte brut Whisper avant nettoyage
- FR32: En mode debug, l'utilisateur peut voir la route choisie (rules-only vs rules+LLM) et la raison
- FR33: Le système peut détecter la présence du modèle Whisper au démarrage
- FR34: Le système peut guider l'utilisateur pour le téléchargement initial du modèle si absent
- FR35: Le système peut détecter la présence d'Ollama et informer l'utilisateur si absent (fallback rules-only)

### NonFunctional Requirements

- NFR1: La latence pipeline complète (fin de parole → texte collé) doit être < 3s au p95 pour une phrase de 15 mots en mode rules-only
- NFR2: La latence pipeline avec LLM (mode Pro) doit être < 5s au p95 pour une phrase de 15 mots
- NFR3: Le temps d'exécution des règles FR doit rester < 1ms au p99 (actuellement ~415 µs)
- NFR4: Le temps de détection VAD doit rester < 1ms au p99 (actuellement ~170 µs)
- NFR5: L'application doit consommer < 4 GB de RAM en fonctionnement (Whisper + LLM chargés)
- NFR6: Le démarrage de l'application (cold start) doit être < 10s incluant le chargement du modèle Whisper
- NFR7: L'application doit avoir un CPU idle < 1% quand aucune dictée n'est active
- NFR8: Aucune donnée audio ou textuelle ne doit quitter la machine sans consentement explicite de l'utilisateur
- NFR9: L'historique local (SQLite) doit être stocké dans le répertoire Application Support de l'utilisateur avec les permissions fichier standard macOS
- NFR10: Les analytics opt-in (si activées) ne doivent contenir aucune donnée personnelle, audio ou textuelle — uniquement des compteurs d'usage anonymisés
- NFR11: Aucun credential, clé API ou token ne doit être stocké en dur dans le code source
- NFR12: Le crash rate doit être < 1% des sessions de dictée pour le MVP, < 0.1% à 12 mois
- NFR13: En cas d'échec du LLM (Ollama indisponible), le système doit basculer silencieusement en mode rules-only sans perte de la dictée en cours
- NFR14: En cas d'erreur de collage curseur, le texte traité doit rester disponible dans le presse-papier et l'historique
- NFR15: L'application doit survivre aux mises en veille / réveil macOS sans perdre sa configuration
- NFR16: L'interface settings doit être navigable entièrement au clavier
- NFR17: L'interface doit être compatible VoiceOver (labels, rôles ARIA) au niveau basique
- NFR18: Le contraste texte/fond doit respecter WCAG 2.1 AA (ratio minimum 4.5:1)
- NFR19: L'application doit fonctionner sur macOS 13 (Ventura) et versions ultérieures
- NFR20: L'application doit fonctionner sur Apple Silicon (M1+) et Intel (avec performance dégradée acceptée)
- NFR21: Le collage curseur doit fonctionner dans les applications standards (navigateurs, éditeurs de texte, suites bureautiques, terminaux)
- NFR22: L'application doit coexister avec d'autres outils utilisant des raccourcis clavier globaux (détection de conflits)
- NFR23: L'interface utilisateur doit être entièrement en français
- NFR24: Les messages d'erreur et d'onboarding doivent être en français avec un ton accessible (pas de jargon technique)

### Additional Requirements

**Architecture :**
- ADR-007 : Architecture hybride Rust + Swift plugins (FFI via @_cdecl + extern "C")
- Budget RAM total cible : ~1.5 Go (Whisper ~800 Mo + LLM ~300 Mo + Tauri ~200 Mo + buffers ~50 Mo)
- Gestion mémoire : chargement modèles à la demande, déchargement après timeout configurable
- Mode "ultra-léger" : STT seul sans LLM (économie ~300 Mo RAM)
- Pipeline async via Tokio channels
- Build whisper.cpp avec WHISPER_COREML=1 + WHISPER_METAL=1 (nécessite Xcode + CMake)
- Distribution : .dmg via GitHub Releases, code signing + notarization Apple
- Auto-update via Tauri Updater (GitHub Releases endpoint)
- CI/CD : benchmarks automatisés à chaque commit (latence p50/p99, zero-edit rate)
- Pas de starter template — fork de Handy existant comme base

**UX Design :**
- Simplification sidebar de 7 sections → 3 (Accueil, Style, Paramètres)
- Palette "cahier de classe" : papier crème #faf8f3, encre verte #2d7a4f, bleue #1a56db, rouge #dc2626
- Typographie : Caveat (branding), Inter (UI), JetBrains Mono (code)
- 4 sons : stylo sur papier (activation), page flip (mode change), wood tap (stop), paper crumple (erreur)
- Overlay enregistrement : capsule 200x40px, 4 états (idle/listening/transcribing/pasting)
- Onboarding 4 étapes : Micro → Accessibility → Modèle → Test dictée
- Menu bar : icône tray avec 3 états visuels (idle gris, recording vert, processing orange)
- Fenêtre settings : 680x480px fixe
- Migration Handy : supprimer HandyTextLogo, TranslateToEnglish, dark theme ; remplacer logo, couleurs, sidebar
- Design tokens définis (spacing, radius, shadows, transitions)
- 9 composants UI réutilisables thématisés "cahier"

### FR Coverage Map

| FR | Epic | Description |
|----|------|-------------|
| FR1 | Epic 1 | Raccourci clavier global |
| FR2 | Epic 1 | Arrêt dictée |
| FR3 | Epic 1 | Capture audio + Whisper FR |
| FR4 | Epic 1 | VAD détection parole |
| FR5 | Epic 1 | Règles nettoyage FR |
| FR6 | Epic 3 | Routing conditionnel LLM |
| FR7 | Epic 1 | Collage curseur |
| FR8 | Epic 3 | Sélection mode Chat/Pro/Code |
| FR9 | Epic 3 | Mode Chat (correction minimale) |
| FR10 | Epic 3 | Mode Pro (reformulation LLM) |
| FR11 | Epic 3 | Mode Code (jargon préservé) |
| FR12 | Epic 2 | Son activation |
| FR13 | Epic 2 | Overlay visuel |
| FR14 | Epic 2 | États dictée |
| FR15 | Epic 4 | Enregistrement sessions |
| FR16 | Epic 4 | Consultation historique |
| FR17 | Epic 4 | Copie depuis historique |
| FR18 | Epic 7 | Raccourci configurable |
| FR19 | Epic 7 | Choix micro |
| FR20 | Epic 3 | Toggle LLM |
| FR21 | Epic 7 | Toggle debug |
| FR22 | Epic 7 | Autostart macOS |
| FR23 | Epic 7 | Accès via menu bar |
| FR24 | Epic 6 | Navigation 3 sections |
| FR25 | Epic 6 | Infos app |
| FR26 | Epic 6 | Thème cahier de classe |
| FR27 | Epic 5 | 100% offline |
| FR28 | Epic 5 | Permissions FR |
| FR29 | Epic 5 | Indicateur réseau |
| FR30 | Epic 7 | Métriques debug |
| FR31 | Epic 7 | Texte brut Whisper |
| FR32 | Epic 7 | Route choisie |
| FR33 | Epic 5 | Détection Whisper |
| FR34 | Epic 5 | Guide téléchargement |
| FR35 | Epic 5 | Détection Ollama |

## Epic List

### Epic 1 : Dictée vocale fondamentale
L'utilisateur parle et le texte propre apparaît au curseur. Le cœur du produit : raccourci global → capture audio → VAD → Whisper FR → règles de nettoyage FR → collage au curseur. Sans LLM, sans UI fancy — juste "ça marche".
**FRs couverts :** FR1, FR2, FR3, FR4, FR5, FR7
**NFRs associés :** NFR1, NFR3, NFR4, NFR5, NFR6, NFR7, NFR12, NFR14, NFR19, NFR20, NFR21

### Epic 2 : Retour visuel et sonore
L'utilisateur sait ce qui se passe pendant sa dictée. Son de stylo à l'activation, overlay capsule pendant l'enregistrement, états visuels (écoute / traitement / terminé). Le feedback qui transforme une boîte noire en expérience fluide.
**FRs couverts :** FR12, FR13, FR14

### Epic 3 : Modes d'écriture intelligents
L'utilisateur choisit comment son texte est écrit (Chat, Pro, Code) avec LLM conditionnel. Sélecteur de mode, prompts dédiés par mode, routing hybride (confiance >= 0.82 → rules-only, sinon → LLM Ollama), toggle activation/désactivation LLM.
**FRs couverts :** FR6, FR8, FR9, FR10, FR11, FR20
**NFRs associés :** NFR2, NFR13

### Epic 4 : Historique des dictées
L'utilisateur retrouve et réutilise ses dictées passées. Enregistrement automatique de chaque session (texte brut, texte traité, horodatage, mode), consultation de l'historique, copie rapide.
**FRs couverts :** FR15, FR16, FR17
**NFRs associés :** NFR9

### Epic 5 : Onboarding, modèles et vie privée
L'utilisateur est guidé au premier lancement, les modèles sont prêts, et il sait que ses données restent chez lui. Flow onboarding 4 étapes (Micro → Accessibility → Modèle → Test), détection Whisper/Ollama, guide téléchargement, fonctionnement 100% offline, indicateur réseau.
**FRs couverts :** FR27, FR28, FR29, FR33, FR34, FR35
**NFRs associés :** NFR8, NFR10, NFR11, NFR24

### Epic 6 : Identité visuelle DictAI
L'interface reflète l'identité "cahier de classe" et est simplifiée pour un public non-technique. Thème papier crème / encre verte, navigation 3 sections (Accueil, Style, Paramètres), infos app, migration visuelle depuis Handy, design tokens.
**FRs couverts :** FR24, FR25, FR26
**NFRs associés :** NFR16, NFR17, NFR18, NFR23

### Epic 7 : Paramètres et personnalisation
L'utilisateur configure l'app selon ses préférences et les dev ont accès au debug. Raccourci configurable, sélection micro, autostart macOS, accès via menu bar, mode debug (métriques pipeline, texte brut, route choisie).
**FRs couverts :** FR18, FR19, FR21, FR22, FR23, FR30, FR31, FR32
**NFRs associés :** NFR15, NFR22

---

## Epic 1 : Dictée vocale fondamentale

L'utilisateur parle et le texte propre apparaît au curseur. Le cœur du produit : raccourci global → capture audio → VAD → Whisper FR → règles de nettoyage FR → collage au curseur.

### Story 1.1 : Démarrer et arrêter une dictée par raccourci clavier

En tant qu'utilisateur,
je veux démarrer et arrêter l'enregistrement via un raccourci clavier global,
afin de dicter sans quitter l'application en cours.

**Acceptance Criteria :**

**Given** l'app DictAI est lancée et le modèle Whisper est chargé
**When** l'utilisateur appuie sur le raccourci global (défaut : Cmd+Shift+D)
**Then** l'enregistrement audio démarre immédiatement
**And** le raccourci fonctionne dans n'importe quelle application au premier plan

**Given** un enregistrement est en cours
**When** l'utilisateur relâche le raccourci (push-to-talk) ou appuie une seconde fois (toggle)
**Then** l'enregistrement s'arrête et le pipeline de transcription se déclenche

**Given** une autre application a le focus
**When** le raccourci global est pressé
**Then** l'app ne vole pas le focus et l'enregistrement démarre en arrière-plan

### Story 1.2 : Transcription vocale en français

En tant qu'utilisateur,
je veux que mon audio soit transcrit en texte français localement,
afin d'obtenir une transcription rapide et privée.

**Acceptance Criteria :**

**Given** un enregistrement audio vient de se terminer
**When** le pipeline de transcription s'exécute
**Then** l'audio est envoyé au VAD (Silero) pour détecter les segments de parole
**And** les segments sont transcrits via Whisper large-v3-turbo Q5_0 en français
**And** la transcription est retournée avec un score de confiance

**Given** un enregistrement audio de 10s en français standard
**When** la transcription est terminée
**Then** la latence STT est < 600ms sur Apple Silicon (p95)
**And** le WER est < 10% sur du français conversationnel

**Given** l'utilisateur parle puis fait une pause de 2s
**When** le VAD détecte la fin de parole
**Then** le temps de détection est < 1ms au p99

**Given** l'application tourne sur une machine avec 6 Go de RAM
**When** Whisper est chargé
**Then** la consommation RAM totale de l'app reste < 1.6 Go

### Story 1.3 : Nettoyage automatique du texte par règles FR

En tant qu'utilisateur,
je veux que le texte transcrit soit automatiquement nettoyé (filler words, ponctuation, bégaiements),
afin d'obtenir un texte prêt à l'emploi sans retouche.

**Acceptance Criteria :**

**Given** une transcription brute contenant des filler words FR ("euh", "du coup", "genre", "voilà", "en fait", etc.)
**When** les règles de nettoyage sont appliquées
**Then** tous les filler words sont supprimés du texte

**Given** une transcription avec des élisions parasites Whisper ("j' ai", "c' est", "l' homme")
**When** les règles de nettoyage sont appliquées
**Then** les espaces parasites après apostrophe sont supprimés ("j'ai", "c'est", "l'homme")

**Given** une transcription avec ponctuation doublée ("..", "??", "!!")
**When** les règles de nettoyage sont appliquées
**Then** la ponctuation est normalisée (simple) et "..." devient "…"

**Given** une transcription avec bégaiements ("je je veux partir")
**When** les règles de nettoyage sont appliquées
**Then** les mots répétés consécutifs sont réduits à un seul

**Given** n'importe quelle transcription
**When** les règles de nettoyage sont appliquées
**Then** le texte résultant commence par une majuscule et se termine par une ponctuation
**And** le temps d'exécution est < 1ms au p99

### Story 1.4 : Collage automatique au curseur actif

En tant qu'utilisateur,
je veux que le texte nettoyé soit collé automatiquement là où se trouve mon curseur,
afin de ne pas avoir à changer d'application ni coller manuellement.

**Acceptance Criteria :**

**Given** le texte nettoyé est prêt et une application texte est au premier plan
**When** le pipeline envoie le texte au module de collage
**Then** le texte est copié dans le presse-papier et collé à la position du curseur via Accessibility API
**And** le collage fonctionne dans Chrome, VS Code, Notion, Mail et Slack

**Given** le collage via Accessibility API échoue
**When** le fallback est activé
**Then** le texte est collé via simulation Cmd+V (CGEvent)
**And** le texte reste disponible dans le presse-papier

**Given** le pipeline complet (raccourci → audio → VAD → Whisper → rules → paste)
**When** on mesure la latence bout-en-bout sur Apple Silicon
**Then** elle est < 3s au p95 pour une phrase de 15 mots en mode rules-only

---

## Epic 2 : Retour visuel et sonore

L'utilisateur sait ce qui se passe pendant sa dictée. Son de stylo à l'activation, overlay capsule pendant l'enregistrement, états visuels (écoute / traitement / terminé).

### Story 2.1 : Son de confirmation à l'activation et à l'arrêt

En tant qu'utilisateur,
je veux entendre un son distinctif quand la dictée démarre et quand elle s'arrête,
afin de savoir sans regarder l'écran que DictAI m'écoute (ou a fini).

**Acceptance Criteria :**

**Given** l'utilisateur active la dictée via le raccourci
**When** l'enregistrement démarre
**Then** un son "stylo sur papier" est joué
**And** le son est < 500ms et ne couvre pas la voix de l'utilisateur

**Given** l'enregistrement se termine (relâchement raccourci ou toggle)
**When** le pipeline de transcription se déclenche
**Then** un son "wood tap" est joué pour confirmer l'arrêt

**Given** l'utilisateur a désactivé le feedback audio dans les paramètres
**When** la dictée démarre ou s'arrête
**Then** aucun son n'est joué

### Story 2.2 : Overlay visuel d'enregistrement avec états

En tant qu'utilisateur,
je veux voir un indicateur visuel pendant ma dictée (écoute, traitement, terminé),
afin de suivre où en est le pipeline sans quitter mon application.

**Acceptance Criteria :**

**Given** l'utilisateur active la dictée
**When** l'enregistrement démarre
**Then** une capsule overlay (200x40px) apparaît en haut de l'écran en vert (#2d7a4f) avec une animation pulse
**And** l'overlay ne vole pas le focus de l'application active

**Given** l'enregistrement est actif
**When** l'utilisateur parle
**Then** l'overlay affiche l'état "Écoute…" avec l'animation d'ondes

**Given** l'enregistrement se termine
**When** le pipeline Whisper/rules se déclenche
**Then** l'overlay passe en état "Traitement…" avec une animation spinning (orange)

**Given** le texte est collé au curseur
**When** le pipeline est terminé
**Then** l'overlay affiche brièvement "Terminé" puis disparaît en fade-out (300ms)

**Given** une erreur survient dans le pipeline
**When** le texte ne peut pas être transcrit ou collé
**Then** l'overlay passe en rouge (#dc2626) avec le message d'erreur pendant 3s

---

## Epic 3 : Modes d'écriture intelligents

L'utilisateur choisit comment son texte est écrit (Chat, Pro, Code) avec LLM conditionnel. Sélecteur de mode, prompts dédiés par mode, routing hybride.

### Story 3.1 : Sélection du mode d'écriture (Chat, Pro, Code)

En tant qu'utilisateur,
je veux choisir un mode d'écriture avant de dicter,
afin que le texte soit formaté selon le contexte (conversation, email pro, documentation technique).

**Acceptance Criteria :**

**Given** l'utilisateur ouvre l'interface DictAI
**When** il accède à la section Style / Accueil
**Then** il voit les 3 modes d'écriture (Chat, Pro, Code) avec une description claire de chacun
**And** le mode actif est visuellement distingué

**Given** l'utilisateur sélectionne un mode
**When** il clique sur Chat, Pro ou Code
**Then** le mode est immédiatement actif et persisté dans les settings
**And** les dictées suivantes utilisent ce mode

**Given** l'utilisateur n'a jamais changé de mode
**When** il lance sa première dictée
**Then** le mode Chat est utilisé par défaut

### Story 3.2 : Mode Chat — correction minimale par règles

En tant qu'utilisateur en mode Chat,
je veux que mon texte soit corrigé légèrement (orthographe, ponctuation) sans changer mon ton,
afin de dicter des messages informels rapidement.

**Acceptance Criteria :**

**Given** le mode Chat est actif et le score de confiance Whisper >= 0.82 et le texte <= 30 mots
**When** la transcription est traitée
**Then** seules les règles FR sont appliquées (pas de LLM)
**And** le ton original est conservé, seuls les filler words et la ponctuation sont corrigés

**Given** le mode Chat est actif et le score de confiance < 0.82 ou le texte > 30 mots
**When** la transcription est traitée
**Then** le LLM est appelé avec le prompt Chat ("Corrige uniquement l'orthographe et ajoute la ponctuation de base. Conserve le ton et la structure. Ne reformule pas.")

**Given** le texte dicté "euh du coup je voulais te dire que c'est super cool"
**When** traité en mode Chat
**Then** le résultat est proche de "Je voulais te dire que c'est super cool." (ton conservé, filler words supprimés)

### Story 3.3 : Mode Pro — reformulation professionnelle via LLM

En tant qu'utilisateur en mode Pro,
je veux que mon texte soit reformulé de manière concise et professionnelle,
afin de dicter des emails et documents avec un ton adapté sans effort.

**Acceptance Criteria :**

**Given** le mode Pro est actif
**When** la transcription est traitée
**Then** le LLM est toujours appelé (quel que soit le score de confiance)
**And** le prompt Pro est utilisé ("Reformule ce texte de manière concise et professionnelle pour un email ou document.")

**Given** le texte dicté "bonjour en fait je voulais vous dire que j'ai bien avancé sur le projet du coup on pourrait se voir la semaine prochaine si ça vous va"
**When** traité en mode Pro
**Then** le résultat est une reformulation professionnelle (ex: "Bonjour, je souhaitais vous informer que j'ai bien progressé sur le projet. Serait-il possible de convenir d'un rendez-vous la semaine prochaine ?")

**Given** Ollama n'est pas disponible
**When** une dictée en mode Pro est lancée
**Then** le système bascule silencieusement en mode rules-only
**And** le texte est quand même retourné (nettoyé par règles uniquement)
**And** l'overlay affiche un avertissement discret

**Given** le LLM est appelé en mode Pro
**When** on mesure la latence
**Then** elle est < 5s au p95 pour une phrase de 15 mots (pipeline complet)

### Story 3.4 : Mode Code — préservation du jargon technique

En tant qu'utilisateur en mode Code,
je veux que le jargon technique anglais, les identifiants et les symboles soient préservés intacts,
afin de dicter de la documentation technique et des commentaires de code.

**Acceptance Criteria :**

**Given** le mode Code est actif
**When** la transcription est traitée
**Then** le LLM est appelé avec le prompt Code ("Corrige la ponctuation. Préserve tous les termes techniques anglais, identifiants et symboles. Ne traduis jamais le jargon technique.")

**Given** le texte dicté "cette fonction prend un DataFrame pandas en entrée filtre les lignes où le champ status est égal à active"
**When** traité en mode Code
**Then** les termes techniques sont préservés et idéalement formatés (DataFrame, status, active)

**Given** le mode Code est actif et le score de confiance >= 0.82 et <= 30 mots
**When** la transcription est traitée
**Then** le routing applique les règles FR d'abord, puis le LLM (le routage "rules-only" ne s'applique pas en mode Code)

---

## Epic 4 : Historique des dictées

L'utilisateur retrouve et réutilise ses dictées passées. Enregistrement automatique de chaque session, consultation de l'historique, copie rapide.

### Story 4.1 : Enregistrement automatique des sessions de dictée

En tant qu'utilisateur,
je veux que chaque dictée soit automatiquement sauvegardée avec son contexte,
afin de pouvoir la retrouver plus tard si j'en ai besoin.

**Acceptance Criteria :**

**Given** une dictée vient de se terminer (texte collé au curseur)
**When** le pipeline est complet
**Then** une entrée est créée dans l'historique SQLite contenant : texte brut Whisper, texte traité final, horodatage, mode d'écriture utilisé, route (rules-only ou rules+LLM)

**Given** l'historique est stocké en SQLite
**When** on vérifie l'emplacement du fichier
**Then** il se trouve dans ~/Library/Application Support/DictAI/
**And** les permissions fichier sont celles standard macOS (lecture/écriture utilisateur uniquement)

**Given** l'utilisateur dicte plusieurs fois dans une session
**When** on consulte l'historique
**Then** chaque dictée est une entrée distincte triée par date décroissante

### Story 4.2 : Consultation et copie depuis l'historique

En tant qu'utilisateur,
je veux consulter mes dictées passées et copier un texte rapidement,
afin de retrouver et réutiliser un passage dicté précédemment.

**Acceptance Criteria :**

**Given** l'utilisateur ouvre la section Historique
**When** la liste se charge
**Then** les dictées passées sont affichées en liste avec : aperçu du texte traité (première ligne), horodatage relatif ("il y a 5 min", "hier"), badge du mode utilisé (Chat/Pro/Code)

**Given** l'utilisateur voit une entrée d'historique
**When** il clique sur le bouton copier
**Then** le texte traité est copié dans le presse-papier
**And** un feedback visuel confirme la copie (toast ou changement d'icône)

**Given** l'utilisateur veut voir le détail d'une entrée
**When** il clique sur l'entrée
**Then** il voit le texte brut Whisper et le texte traité côte à côte

**Given** l'historique contient beaucoup d'entrées
**When** l'utilisateur scrolle
**Then** la liste est paginée ou en scroll infini sans lag perceptible

---

## Epic 5 : Onboarding, modèles et vie privée

L'utilisateur est guidé au premier lancement, les modèles sont prêts, et il sait que ses données restent chez lui.

### Story 5.1 : Détection et téléchargement du modèle Whisper

En tant qu'utilisateur,
je veux que DictAI détecte si le modèle de reconnaissance vocale est présent et me guide pour le télécharger si besoin,
afin de pouvoir commencer à dicter sans configuration technique.

**Acceptance Criteria :**

**Given** l'application démarre pour la première fois
**When** le modèle Whisper large-v3-turbo Q5_0 n'est pas trouvé localement
**Then** un écran de téléchargement s'affiche avec une barre de progression
**And** le message explique en français simple : "Téléchargement du modèle de reconnaissance vocale (~1.5 Go)…"
**And** aucun jargon technique (pas de "Whisper", "GGML", "Q5")

**Given** le téléchargement du modèle est en cours
**When** l'utilisateur ferme l'application
**Then** le téléchargement reprend là où il s'était arrêté au prochain lancement

**Given** le modèle est déjà présent localement
**When** l'application démarre
**Then** le modèle est détecté et chargé sans afficher l'écran de téléchargement
**And** le cold start est < 10s incluant le chargement

**Given** le téléchargement échoue (réseau indisponible)
**When** l'erreur est affichée
**Then** le message est en français ("Connexion internet nécessaire pour le premier téléchargement. Réessayez plus tard.")
**And** un bouton "Réessayer" est visible

### Story 5.2 : Détection d'Ollama et fallback rules-only

En tant qu'utilisateur,
je veux que DictAI détecte si Ollama est installé et fonctionne sans même si ce n'est pas le cas,
afin de ne pas être bloqué par une dépendance technique.

**Acceptance Criteria :**

**Given** l'application démarre
**When** Ollama est installé et actif
**Then** les modes Pro et Code utilisent le LLM normalement
**And** aucun message particulier n'est affiché

**Given** l'application démarre
**When** Ollama n'est pas détecté
**Then** l'app fonctionne en mode rules-only pour toutes les dictées
**And** un bandeau discret informe : "Pour des corrections avancées, installez Ollama (gratuit)."
**And** un lien vers le site Ollama est fourni

**Given** Ollama était actif et tombe pendant une session
**When** une dictée en mode Pro est lancée
**Then** le système bascule silencieusement en rules-only
**And** le texte est quand même retourné sans perte
**And** l'overlay affiche brièvement "Mode simplifié (Ollama indisponible)"

**Given** l'app fonctionne sans Ollama
**When** l'utilisateur utilise le mode Chat avec score de confiance >= 0.82
**Then** l'expérience est identique (les règles FR suffisent)

### Story 5.3 : Onboarding permissions macOS

En tant qu'utilisateur au premier lancement,
je veux être guidé pour accorder les permissions nécessaires avec des explications claires,
afin de comprendre pourquoi chaque permission est demandée et me sentir en confiance.

**Acceptance Criteria :**

**Given** l'application est lancée pour la première fois
**When** l'onboarding démarre
**Then** un écran explique la permission Microphone en français : "DictAI a besoin d'accéder à votre micro pour transcrire votre voix. Votre audio est traité localement et n'est jamais envoyé sur internet."
**And** un bouton "Autoriser le microphone" déclenche la demande système macOS

**Given** la permission Microphone est accordée
**When** l'onboarding passe à l'étape suivante
**Then** un écran explique la permission Accessibility : "DictAI a besoin de l'accès Accessibilité pour coller le texte directement dans vos applications."
**And** un bouton ouvre les Préférences Système à la bonne section

**Given** une permission est refusée
**When** l'utilisateur revient dans l'app
**Then** un message explique ce qui ne fonctionnera pas sans cette permission
**And** un bouton permet de rouvrir les Préférences Système

**Given** toutes les permissions sont accordées
**When** l'onboarding se termine
**Then** un écran "Test rapide" propose de faire une première dictée test
**And** le texte de la dictée test apparaît dans l'interface pour confirmer que tout fonctionne

### Story 5.4 : Indicateur de vie privée

En tant qu'utilisateur,
je veux pouvoir vérifier qu'aucune donnée ne quitte ma machine,
afin d'avoir confiance dans la promesse de confidentialité de DictAI.

**Acceptance Criteria :**

**Given** l'application fonctionne normalement (100% offline)
**When** l'utilisateur consulte l'indicateur de statut réseau
**Then** un badge vert "Local" est visible, confirmant qu'aucune connexion réseau n'est utilisée

**Given** une connexion réseau est active (téléchargement modèle ou vérification mise à jour)
**When** l'utilisateur consulte l'indicateur
**Then** le badge indique "Téléchargement en cours" avec le détail de ce qui est transféré
**And** aucune donnée audio ou textuelle n'est transmise

**Given** l'utilisateur est en mode avion (wifi désactivé)
**When** il utilise toutes les fonctionnalités de dictée
**Then** tout fonctionne normalement sans erreur ni dégradation
**And** l'indicateur reste vert "Local"

---

## Epic 6 : Identité visuelle DictAI

L'interface reflète l'identité "cahier de classe" et est simplifiée pour un public non-technique. Thème papier crème / encre verte, navigation 3 sections, migration depuis Handy.

### Story 6.1 : Thème "cahier de classe" et design tokens

En tant qu'utilisateur,
je veux que l'interface ait une identité visuelle chaleureuse et distincte (papier crème, encre verte),
afin de me sentir dans un outil français pensé pour moi, pas dans un fork d'un outil américain.

**Acceptance Criteria :**

**Given** l'utilisateur ouvre l'application
**When** l'interface se charge
**Then** le fond principal est papier crème (#faf8f3)
**And** la couleur primaire est encre verte (#2d7a4f)
**And** les accents secondaires utilisent encre bleue (#1a56db) et rouge correction (#dc2626)
**And** le texte principal est encre sombre (#1a1a2e)

**Given** les design tokens sont appliqués
**When** on inspecte les composants UI
**Then** la typographie branding utilise Caveat (script/manuscrit)
**And** la typographie UI utilise Inter (sans-serif lisible)
**And** la typographie code utilise JetBrains Mono

**Given** le thème "cahier de classe" est appliqué
**When** on vérifie le contraste texte/fond
**Then** tous les textes respectent WCAG 2.1 AA (ratio minimum 4.5:1)

**Given** l'ancien thème Handy (sombre, logo main)
**When** la migration visuelle est effectuée
**Then** le logo DictationLogo remplace HandyTextLogo
**And** les composants HandyHand, HandyTextLogo, TranslateToEnglish sont supprimés
**And** le dark theme Handy est retiré

### Story 6.2 : Navigation simplifiée 3 sections

En tant qu'utilisateur non-technique,
je veux une navigation simple avec seulement 3 sections (Accueil, Style, Paramètres),
afin de ne pas être submergé par des options techniques que je ne comprends pas.

**Acceptance Criteria :**

**Given** l'utilisateur ouvre l'application
**When** la sidebar se charge
**Then** elle affiche exactement 3 sections : Accueil, Style, Paramètres
**And** les 7 sections Handy (general, models, advanced, postprocessing, history, debug, about) sont réorganisées dans ces 3 sections

**Given** l'utilisateur clique sur "Accueil"
**When** la section se charge
**Then** il voit : l'état de la dictée, le sélecteur de mode rapide, un aperçu de la dernière dictée

**Given** l'utilisateur clique sur "Style"
**When** la section se charge
**Then** il voit : les 3 cartes de mode (Chat, Pro, Code) avec descriptions et exemples avant/après

**Given** l'utilisateur clique sur "Paramètres"
**When** la section se charge
**Then** il voit les réglages groupés en accordéons : Audio, Raccourcis, Modèles, Avancé, À propos
**And** le panneau Debug est masqué par défaut (accessible uniquement si activé dans Avancé)

**Given** la sidebar est affichée
**When** l'utilisateur navigue au clavier (Tab, flèches)
**Then** toutes les sections sont accessibles au clavier
**And** le focus est visible et suit les standards VoiceOver

### Story 6.3 : Page À propos

En tant qu'utilisateur,
je veux voir les informations de l'application (version, licence, crédits),
afin de savoir quelle version j'utilise et qui a créé DictAI.

**Acceptance Criteria :**

**Given** l'utilisateur va dans Paramètres > À propos
**When** la section se charge
**Then** il voit : le nom "DictAI", le numéro de version (semver), la licence MIT, le lien GitHub, les crédits (fork de Handy, Whisper, Ollama)

**Given** l'utilisateur veut vérifier les mises à jour
**When** il consulte la page À propos
**Then** il voit si une mise à jour est disponible (si le check auto-update est activé)

**Given** toute l'interface (y compris À propos)
**When** on vérifie la langue
**Then** tous les textes, labels et messages sont en français
**And** aucun terme technique anglais n'apparaît dans l'UI visible (hors panneau debug)

---

## Epic 7 : Paramètres et personnalisation

L'utilisateur configure l'app selon ses préférences et les dev ont accès au debug. Raccourci configurable, sélection micro, autostart macOS, accès via menu bar, mode debug.

### Story 7.1 : Configuration du raccourci clavier

En tant qu'utilisateur,
je veux pouvoir changer le raccourci clavier de dictée,
afin d'utiliser une combinaison qui ne rentre pas en conflit avec mes autres outils.

**Acceptance Criteria :**

**Given** l'utilisateur va dans Paramètres > Raccourcis
**When** il clique sur le champ de raccourci
**Then** le champ passe en mode écoute et capture la prochaine combinaison de touches pressée
**And** le nouveau raccourci est affiché et sauvegardé immédiatement

**Given** l'utilisateur saisit un raccourci déjà utilisé par une autre application
**When** le conflit est détecté
**Then** un avertissement est affiché : "Ce raccourci peut entrer en conflit avec [nom de l'app si détectable]."
**And** l'utilisateur peut quand même confirmer ou en choisir un autre

**Given** l'utilisateur a configuré un raccourci personnalisé
**When** il redémarre l'application
**Then** le raccourci personnalisé est restauré (persisté dans les settings)

**Given** l'utilisateur veut revenir au raccourci par défaut
**When** il clique sur "Réinitialiser"
**Then** le raccourci revient à Cmd+Shift+D

### Story 7.2 : Sélection du microphone et préférences audio

En tant qu'utilisateur,
je veux choisir quel microphone utiliser pour la dictée,
afin d'utiliser mon micro externe de meilleure qualité plutôt que le micro intégré.

**Acceptance Criteria :**

**Given** l'utilisateur va dans Paramètres > Audio
**When** la section se charge
**Then** une liste déroulante affiche tous les microphones disponibles sur la machine
**And** le micro actuellement sélectionné est mis en évidence

**Given** l'utilisateur sélectionne un micro différent
**When** il choisit dans la liste
**Then** le micro est immédiatement actif pour la prochaine dictée
**And** le choix est persisté dans les settings

**Given** un micro externe est débranché pendant l'utilisation
**When** le micro sélectionné n'est plus disponible
**Then** le système bascule sur le micro par défaut du système
**And** un message discret informe l'utilisateur du changement

**Given** l'application survit à une mise en veille / réveil macOS
**When** l'utilisateur reprend la dictée
**Then** le micro sélectionné est toujours actif et la configuration est intacte

### Story 7.3 : Accès via menu bar et autostart

En tant qu'utilisateur,
je veux accéder à DictAI depuis l'icône dans la barre de menu et optionnellement lancer l'app au démarrage,
afin que DictAI soit toujours disponible sans encombrer mon dock.

**Acceptance Criteria :**

**Given** l'application est lancée
**When** l'icône apparaît dans la barre de menu macOS
**Then** l'icône reflète l'état : gris (idle), vert (enregistrement), orange (traitement)

**Given** l'utilisateur clique sur l'icône menu bar
**When** le menu contextuel s'ouvre
**Then** il voit : le mode actif, "Ouvrir les paramètres", "Quitter DictAI"
**And** un clic sur "Ouvrir les paramètres" ouvre la fenêtre settings (680x480px)

**Given** l'utilisateur active "Lancer au démarrage" dans Paramètres > Avancé
**When** il redémarre son Mac
**Then** DictAI se lance automatiquement en arrière-plan (icône menu bar visible, pas de fenêtre)

**Given** l'utilisateur désactive "Lancer au démarrage"
**When** il redémarre son Mac
**Then** DictAI ne se lance pas automatiquement

### Story 7.4 : Mode debug — métriques et diagnostic pipeline

En tant que développeur ou utilisateur avancé,
je veux activer un mode debug qui affiche les métriques détaillées du pipeline,
afin de diagnostiquer les problèmes de performance ou de qualité de transcription.

**Acceptance Criteria :**

**Given** l'utilisateur va dans Paramètres > Avancé
**When** il active le toggle "Mode debug"
**Then** un panneau debug devient accessible dans l'interface
**And** le panneau n'est visible que si le mode debug est activé

**Given** le mode debug est actif et une dictée est effectuée
**When** l'utilisateur consulte le panneau debug
**Then** il voit les métriques de la dernière dictée : temps VAD (ms), temps Whisper (ms), score de confiance Whisper, route choisie (rules-only / rules+LLM) et la raison, temps rules (ms), temps LLM (ms si appelé), temps total pipeline (ms)

**Given** le mode debug est actif
**When** l'utilisateur consulte une entrée de dictée
**Then** il voit le texte brut Whisper (avant nettoyage) à côté du texte traité final
**And** la route choisie est affichée avec la raison (ex: "rules-only : confiance 0.91 >= 0.82, 12 mots <= 30, mode Chat")

**Given** le mode debug est désactivé
**When** l'utilisateur navigue dans l'interface
**Then** aucune information technique n'est visible
**And** le panneau debug est complètement masqué
