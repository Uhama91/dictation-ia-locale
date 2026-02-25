---
stepsCompleted: ['step-01-init', 'step-02-discovery', 'step-02b-vision', 'step-02c-executive-summary', 'step-03-success', 'step-04-journeys', 'step-05-domain', 'step-06-innovation', 'step-07-project-type', 'step-08-scoping', 'step-09-functional', 'step-10-nonfunctional', 'step-11-polish', 'step-12-complete']
status: 'complete'
completedAt: '2026-02-25'
inputDocuments:
  - 'docs/specs/functional-spec.md'
  - 'docs/project-structure.md'
  - 'docs/technical-review-final.md'
  - 'docs/adr/001-backend-language.md'
  - 'implementation-artifacts/tech-spec-dictation-ia-locale-mvp.md'
  - 'party-mode-session-notes (inline)'
  - 'user-research-notes-mobile-vision (inline)'
workflowType: 'prd'
documentCounts:
  briefs: 0
  research: 0
  projectDocs: 5
  userNotes: 2
---

# Product Requirements Document - DictAI

**Author:** Ullie
**Date:** 2026-02-25

## Executive Summary

DictAI est une application de dictée vocale intelligente, **locale par défaut**, conçue spécifiquement pour le français. L'utilisateur parle naturellement ; DictAI transcrit, nettoie et formate le texte en temps réel, puis le colle automatiquement au curseur actif. Pas de compte cloud obligatoire, pas de données vocales qui quittent la machine.

Le produit cible un public francophone non-technique — freelances, rédacteurs, étudiants, professionnels — qui veut dicter du texte propre sans configurer quoi que ce soit. L'expérience visée : appuyer sur un raccourci, parler, voir le texte apparaître immédiatement, formaté et prêt à l'emploi.

Le problème de fond : les outils de dictée FR actuels sont soit basiques (dictée native Apple/Google, sans intelligence de formatage), soit anglophones et cloud-only (Wispr Flow, Otter.ai). Aucune solution ne combine transcription intelligente, post-traitement par LLM local, et respect de la vie privée — en français.

### What Makes This Special

- **Local-first, privacy-first** — Whisper large-v3-turbo Q5 + Qwen2.5-0.5B tournent entièrement sur la machine. Zéro envoi de données vocales au cloud par défaut.
- **Pensé pour le français** — Règles de nettoyage FR natives (élisions Whisper, filler words francophones, ponctuation FR), pas une traduction d'un produit US.
- **Fluidité perçue** — Streaming par chunks courts (1-2s) pour que le texte apparaisse pendant que l'utilisateur parle, comme Wispr Flow.
- **Smart Formatting** — Le LLM local structure le texte (paragraphes, ponctuation intelligente, citations) au-delà de la simple transcription.
- **Identité forte** — Branding "cahier de classe" (papier crème, encre verte, son de stylo sur papier), nom DictAI (jeu de mots Dict + AI).
- **Open-core** — Desktop gratuit et open-source (MIT). Mobile gratuit. Seule la synchronisation cross-device est payante (4,99 EUR/mois).

## Project Classification

| Dimension | Valeur |
|-----------|--------|
| **Type** | Application Desktop + Mobile (multi-plateforme native) |
| **Domaine** | Productivité / Accessibilité |
| **Complexité** | Medium — pipeline STT→LLM existant, mais multi-plateforme et streaming à intégrer |
| **Contexte** | Brownfield — fork de Handy (Tauri 2.x / Rust / React), rebranding et simplification |
| **Plateformes** | macOS (Rust/Tauri), Android (Kotlin natif), iOS (Swift natif, futur) |
| **Backend sync** | Supabase (free tier initial) |

## Success Criteria

### User Success

- **Zéro friction au démarrage** — L'utilisateur installe, appuie sur le raccourci, et dicte. Pas de compte, pas de configuration Whisper manuelle, pas de téléchargement de modèle visible.
- **Texte propre dès la première dictée** — Le texte collé est directement utilisable : majuscules, ponctuation, filler words supprimés, élisions corrigées. L'utilisateur ne retouche pas plus de 1-2 mots par paragraphe.
- **Fluidité perçue** — Le texte apparaît pendant que l'utilisateur parle (streaming chunks 1-2s). Le délai perçu entre fin de parole et collage final < 2s.
- **Moment "aha!"** — L'utilisateur réalise qu'il dicte plus vite qu'il ne tape, et que le texte est mieux structuré que ce qu'il aurait tapé.
- **Confiance vie privée** — L'utilisateur sait que rien ne quitte sa machine. Pas de doute, pas de disclaimer ambigu.

### Business Success

- **3 mois post-lancement desktop :**
  - 500+ stars GitHub
  - 200+ utilisateurs actifs hebdomadaires (mesurés par opt-in analytics anonymes)
  - 5+ articles/mentions dans la communauté francophone dev/productivité
- **6 mois (lancement Android) :**
  - 1 000+ téléchargements Play Store
  - Taux de rétention J7 > 30%
- **12 mois :**
  - 100+ abonnés sync (4,99 EUR/mois) = ~500 EUR MRR
  - Communauté active (issues, PRs, discussions GitHub)
- **Signal qualitatif :** Au moins 3 retours utilisateurs spontanés du type "je ne peux plus m'en passer"

### Technical Success

- **Latence pipeline complète** (audio → texte collé) : p95 < 3s pour une phrase de 15 mots
- **Précision transcription FR** : WER < 10% sur du français conversationnel standard
- **RAM** : < 4 GB en fonctionnement (Whisper + LLM chargés)
- **Crash rate** : < 0.1% des sessions de dictée
- **Taille app** : < 2 GB installée (modèles inclus)
- **35/35 tests pipeline rules** passent en CI (déjà atteint)

### Measurable Outcomes

| Métrique | Cible MVP | Cible 12 mois |
|----------|-----------|---------------|
| Latence p95 (phrase 15 mots) | < 3s | < 1.5s (streaming) |
| WER français conversationnel | < 10% | < 7% |
| Stars GitHub | 100 | 500+ |
| Utilisateurs actifs hebdo | 50 | 500+ |
| Abonnés sync | — | 100+ |
| Crash rate | < 1% | < 0.1% |

## User Journeys

### Journey 1 : Sophie, rédactrice freelance — Premier contact

**Sophie**, 34 ans, rédactrice web freelance à Lyon. Elle rédige 3-4 articles par jour et a des tendinites récurrentes aux poignets. Elle a essayé la dictée Apple mais passait autant de temps à corriger qu'à taper.

**Opening Scene** — Sophie découvre DictAI sur un post Reddit r/france. "Dictée vocale intelligente, locale, gratuite." Elle télécharge le .dmg, l'installe en 30 secondes. Pas de compte à créer. L'icône apparaît dans la menu bar.

**Rising Action** — Elle appuie sur le raccourci (Cmd+Shift+D). Un son doux de stylo sur papier confirme que DictAI écoute. L'overlay vert apparaît. Elle dicte : "Euh, du coup, j'ai testé ce nouveau restaurant à Lyon, c'est vraiment pas mal genre les pâtes étaient excellentes." Elle relâche le raccourci.

**Climax** — En moins de 2 secondes, le texte apparaît dans son Google Doc : "J'ai testé ce nouveau restaurant à Lyon. C'est vraiment pas mal, les pâtes étaient excellentes." Les "euh", "du coup", "genre" ont disparu. Ponctuation ajoutée. Majuscules en place. Sophie n'a rien à retoucher.

**Resolution** — En une semaine, Sophie dicte 80% de ses articles. Ses poignets la remercient. Elle tweete : "DictAI m'a changé la vie, j'écris 2x plus vite et le texte est propre direct."

> **Capabilities révélées** : Installation zero-config, raccourci global, overlay feedback, pipeline rules FR, collage curseur automatique, son d'activation.

### Journey 2 : Marc, développeur backend — Mode Code

**Marc**, 28 ans, dev backend Python à Bordeaux. Il documente mal son code parce qu'il déteste taper des docstrings. Il veut dicter ses commentaires et documentation technique.

**Opening Scene** — Marc installe DictAI et sélectionne le mode "Code" dans les paramètres. Il ouvre VS Code, place son curseur au-dessus d'une fonction.

**Rising Action** — Il active DictAI et dicte : "Cette fonction prend un DataFrame pandas en entrée, filtre les lignes où le champ status est égal à active, et retourne un nouveau DataFrame trié par date de création décroissante."

**Climax** — DictAI colle : "Cette fonction prend un DataFrame pandas en entrée, filtre les lignes où le champ `status` est égal à `active`, et retourne un nouveau DataFrame trié par date de création décroissante." Le jargon technique est préservé, les termes code sont en backticks.

**Resolution** — Marc documente désormais chaque fonction en la dictant. Son équipe remarque que la couverture documentaire a doublé. Il contribue au repo GitHub avec des suggestions d'amélioration du mode Code.

> **Capabilities révélées** : Mode Code (jargon préservé, formatage Markdown), sélection de mode, intégration multi-app (VS Code, terminal).

### Journey 3 : Sophie (edge case) — Dictée longue et interruption

**Sophie** dicte un article de 800 mots sur la gastronomie lyonnaise. À mi-chemin, son chat renverse un verre. Elle s'interrompt brusquement : "Oh non, le chat !"

**Rising Action** — DictAI détecte la fin de parole via VAD et transcrit la phrase en cours. "Oh non, le chat !" est inclus dans la transcription. Sophie regarde l'historique dans DictAI, retrouve sa dernière dictée.

**Climax** — Elle supprime manuellement "Oh non, le chat !" dans son document, puis reprend la dictée là où elle s'était arrêtée. L'historique local lui montre le texte exact de chaque session de dictée avec horodatage.

**Resolution** — Sophie apprend à faire des dictées de 2-3 paragraphes max plutôt qu'un article entier d'un coup. Elle utilise l'historique pour retrouver des passages dictés précédemment.

> **Capabilities révélées** : Historique local (SQLite), horodatage des sessions, VAD (fin de parole), gestion des interruptions, récupération de texte.

### Journey 4 : Thomas, étudiant en droit — Mode Pro et email

**Thomas**, 22 ans, étudiant en Master 2 Droit à Paris. Il doit rédiger des emails formels à ses professeurs et des notes de synthèse. Il écrit comme il parle — trop familier pour le contexte académique.

**Opening Scene** — Thomas sélectionne le mode "Pro" avant de dicter un email à son directeur de mémoire.

**Rising Action** — Il dicte : "Bonjour professeur, en fait je voulais vous dire que j'ai bien avancé sur mon mémoire, du coup je pensais qu'on pourrait se voir la semaine prochaine pour en discuter si ça vous va."

**Climax** — DictAI (mode Pro, LLM activé) colle : "Bonjour Professeur, je souhaitais vous informer que j'ai bien progressé sur mon mémoire. Serait-il possible de convenir d'un rendez-vous la semaine prochaine pour en discuter ?" Le ton est professionnel, les filler words disparus, la structure reformulée.

**Resolution** — Thomas utilise DictAI pour tous ses emails académiques. Il garde le mode Chat pour ses messages persos. Le basculement entre modes devient un réflexe.

> **Capabilities révélées** : Mode Pro (reformulation LLM), basculement de modes, LLM conditionnel (activé pour Pro), ton professionnel adaptatif.

### Journey 5 : Ullie (créateur/admin) — Configuration et debug

**Ullie**, créateur de DictAI, teste une nouvelle version du pipeline. Il active le mode debug dans les paramètres.

**Opening Scene** — Ullie ouvre DictAI, va dans Paramètres > Debug. Il active le mode debug et le logging détaillé.

**Rising Action** — Il dicte une phrase test. Le panneau debug affiche : temps VAD (0.2ms), temps Whisper (1.8s), confiance (0.87), route (rules-only), temps rules (0.4ms), temps total (1.85s). Il voit que la confiance est au-dessus du seuil 0.82, donc le LLM n'a pas été appelé.

**Climax** — Il force une dictée en mode Pro pour tester le chemin LLM. Le debug montre : route (rules+LLM), temps Ollama (890ms), prompt envoyé, réponse reçue. Il identifie un goulot d'étranglement dans le chargement du modèle à froid.

**Resolution** — Il corrige le cold-start en préchargeant le modèle au démarrage de l'app. La latence LLM passe de 890ms à 340ms. Il commit le fix.

> **Capabilities révélées** : Mode debug, logging pipeline détaillé, métriques de performance, basculement conditionnel rules/LLM visible, configuration avancée.

### Journey Requirements Summary

| Journey | Capabilities clés |
|---------|-------------------|
| Sophie (premier contact) | Installation zero-config, raccourci global, overlay, pipeline rules FR, son d'activation, collage curseur |
| Marc (mode Code) | Mode Code, jargon préservé, formatage Markdown, sélection de mode |
| Sophie (edge case) | Historique local, VAD, horodatage sessions, gestion interruptions |
| Thomas (mode Pro) | Mode Pro, reformulation LLM, basculement modes, ton professionnel |
| Ullie (admin/debug) | Mode debug, métriques pipeline, logging détaillé, configuration avancée |

## Domain-Specific Requirements

### Privacy & Data Handling

- **RGPD** — L'app traite des données vocales (données personnelles au sens RGPD). Le traitement 100% local signifie pas de transfert de données, mais il faut quand même :
  - Informer l'utilisateur que l'audio est traité localement et jamais envoyé
  - Pas de collecte de données vocales, même pour améliorer le modèle, sauf consentement explicite
  - L'historique local (SQLite) reste sous contrôle exclusif de l'utilisateur
- **Opt-in analytics** — Toute télémétrie (usage, crash reports) doit être opt-in explicite, jamais par défaut

### Accessibility (macOS)

- **Permissions macOS** — L'app nécessite : Microphone, Accessibility (collage curseur via AppleScript/AX API). Ces permissions doivent être demandées avec des explications claires en français.
- **VoiceOver** — L'interface settings doit être navigable au clavier et compatible VoiceOver (a11y basique)

### Technical Constraints

- **Taille modèle** — Whisper large-v3-turbo Q5 (~1.5 GB) doit être bundlé ou téléchargé au premier lancement. UX de téléchargement nécessaire si non bundlé.
- **Apple Silicon prioritaire** — Optimisation ANE (Apple Neural Engine) pour Whisper sur M1/M2/M3. Support Intel en fallback CPU mais performance dégradée attendue.
- **Ollama dependency** — Le LLM local nécessite Ollama installé. Soit bundler, soit guider l'installation, soit fallback rules-only si absent.

### Risk Mitigations

| Risque | Impact | Mitigation |
|--------|--------|------------|
| Utilisateur sans Ollama | Pas de mode Pro/Smart Formatting | Fallback rules-only avec message clair |
| Mac Intel < 16 GB RAM | Lenteur ou crash | Détection hardware, message d'avertissement, suggestion modèle plus léger |
| Permissions macOS refusées | App non fonctionnelle | Écran onboarding expliquant chaque permission avec sa raison |
| Audio ambiant bruyant | Transcription dégradée | VAD stricte + message "environnement bruyant détecté" |

## Innovation & Novel Patterns

### Detected Innovation Areas

1. **Pipeline hybride rules + LLM local** — Aucun outil de dictée grand public ne combine des règles regex compilées (< 1ms) avec un fallback LLM local conditionnel basé sur un score de confiance. Ce routage intelligent (confiance >= 0.82 → rules only, sinon → rules + LLM) est une approche originale qui optimise latence ET qualité.

2. **Première solution de dictée intelligente FR, locale et open-source** — Il n'existe pas de concurrent direct qui soit à la fois : francophone natif, local-first, intelligent (post-traitement LLM), et open-source. Wispr Flow est cloud-only et anglophone. La dictée Apple/Google est locale mais sans intelligence de formatage.

3. **Identité UX "cahier de classe"** — Dans un marché dominé par des UI tech minimalistes (Wispr Flow = noir/blanc épuré), le choix d'un branding nostalgique (papier crème, encre verte, son de stylo, boucles scripturales) crée une différenciation émotionnelle forte et un ancrage culturel francophone.

### Market Context & Competitive Landscape

| Solution | Local | FR natif | LLM post-traitement | Open-source | Prix |
|----------|-------|----------|---------------------|-------------|------|
| **DictAI** | Oui | Oui | Oui (local) | Oui | Gratuit + sync 4,99 EUR |
| Wispr Flow | Non (cloud) | Non (EN) | Oui (cloud) | Non | $8/mois |
| Dictée Apple | Oui | Partiel | Non | Non | Gratuit |
| Dictée Google | Non (cloud) | Partiel | Non | Non | Gratuit |
| Otter.ai | Non (cloud) | Non (EN) | Oui (cloud) | Non | $16.99/mois |

**Positionnement unique** : DictAI est le seul à cocher les 4 premières colonnes simultanément.

### Validation Approach

- **MVP desktop** comme preuve de concept — valider que le pipeline local atteint une qualité suffisante pour un usage quotidien sans cloud
- **Beta francophone ouverte** — recueillir des retours sur la qualité FR spécifiquement (élisions, filler words, jargon)
- **Benchmark WER** — comparer objectivement Whisper large-v3-turbo Q5 local vs. API cloud sur un corpus FR standardisé
- **Mesure de rétention** — le signal clé est la rétention J7 > 30% (l'utilisateur revient après une semaine)

### Risk Mitigation

| Innovation | Risque | Fallback |
|------------|--------|----------|
| Pipeline hybride rules+LLM | Le LLM local est trop lent ou imprécis | Mode rules-only performant, cloud opt-in |
| 100% local | Qualité inférieure au cloud | Option cloud explicite (API OpenAI/Anthropic) en opt-in |
| Branding "cahier" | Perçu comme enfantin ou non-sérieux | A/B test visuel, thème alternatif neutre disponible |
| Open-source | Pas de moat commercial | Le moat est la communauté FR + la sync comme seul paywall |

## Desktop App Specific Requirements

### Project-Type Overview

DictAI est une **desktop app native macOS** (Tauri 2.x / Rust backend + React frontend) fonctionnant en mode menu bar. L'app est 100% offline par design — aucune connexion réseau requise pour le core pipeline. Le cross-platform (Windows/Linux) n'est pas prévu au MVP mais Tauri le permet à terme.

### Platform Support

| Plateforme | Statut | Notes |
|------------|--------|-------|
| **macOS (Apple Silicon)** | MVP | Cible principale. ANE pour Whisper. M1/M2/M3/M4. |
| **macOS (Intel)** | MVP (dégradé) | Fallback CPU. Performance réduite. Warning à l'install. |
| **Windows** | Futur | Tauri supporte Windows. Post-MVP si demande communauté. |
| **Linux** | Futur | Tauri supporte Linux. Post-MVP si demande communauté. |

### System Integration

- **Menu bar** — App réside dans la barre de menu macOS (pas de fenêtre dock). Clic = ouvre le panneau settings/overlay.
- **Raccourci clavier global** — Enregistré via macOS Accessibility API. Configurable par l'utilisateur (défaut : Cmd+Shift+D).
- **Collage curseur** — Via AppleScript / Accessibility API (AX). Colle le texte transcrit dans l'app active au curseur.
- **Permissions macOS requises** :
  - `NSMicrophoneUsageDescription` — Capture audio pour transcription
  - `AXIsProcessTrusted` — Accessibility pour raccourci global + collage curseur
- **Autostart** — Option pour lancer DictAI au démarrage macOS (login items).

### Update Strategy

- **Auto-update via Tauri Updater** — Vérifie les mises à jour au lancement (endpoint GitHub Releases).
- **Mise à jour silencieuse** — Télécharge en arrière-plan, applique au prochain redémarrage.
- **Versioning sémantique** — `MAJOR.MINOR.PATCH` sur GitHub Releases.
- **Modèles Whisper** — Gérés séparément (pas dans l'updater app). Téléchargement initial au premier lancement ou bundlé dans le .dmg.

### Offline Capabilities

- **100% offline par défaut** — Le core pipeline (audio → Whisper → rules → collage) fonctionne sans connexion.
- **Ollama local** — Le LLM tourne localement via Ollama. Aucun appel réseau.
- **Historique local** — SQLite embarqué, stocké dans `~/Library/Application Support/DictAI/`.
- **Seule connexion optionnelle** : auto-update check + future sync Supabase (opt-in).

### Implementation Considerations

- **Taille du bundle** — Le .dmg sans modèle Whisper pèse ~30 MB. Avec le modèle bundlé : ~1.5 GB. Deux options de distribution à évaluer.
- **Premier lancement UX** — Si modèle non bundlé : écran de téléchargement avec barre de progression et explication ("Téléchargement du modèle de reconnaissance vocale...").
- **Swift plugins** — Fonctionnalités macOS-spécifiques (ANE, MenuBar, AccessibilityPaste) implémentées en Swift via Tauri plugin bridge.
- **Code signing** — Nécessaire pour distribution hors Mac App Store. Certificat développeur Apple requis pour éviter Gatekeeper warnings.
- **Notarization Apple** — Requise pour macOS Ventura+ pour éviter les alertes de sécurité. Process à intégrer en CI/CD.

## Project Scoping & Phased Development

### MVP Strategy & Philosophy

**MVP Approach :** Experience MVP — démontrer que la dictée vocale locale peut être aussi fluide et propre que les solutions cloud, en français. L'objectif n'est pas de lancer un produit complet mais de valider une expérience : "je parle, le texte propre apparaît instantanément".

**Resource Requirements :** Solo dev (Ullie). Stack existante (Tauri/Rust fork de Handy) réduit considérablement le temps de développement. Estimation : 4-6 semaines pour MVP fonctionnel.

### MVP Feature Set (Phase 1)

**Core User Journeys Supported :**
- Sophie (premier contact) — installation → dictée → texte propre collé
- Thomas (mode Pro) — basculement de mode + reformulation LLM
- Marc (mode Code) — jargon préservé

**Must-Have Capabilities :**

| Capability | Justification | Statut |
|------------|---------------|--------|
| Raccourci clavier global | Sans ça, pas de produit | Implémenté |
| Capture audio microphone | Entrée du pipeline | Implémenté |
| Whisper large-v3-turbo Q5 (STT) | Core transcription | Implémenté |
| Rules FR (filler words, élisions, ponctuation) | Qualité texte minimale | Implémenté (35 tests) |
| VAD (Voice Activity Detection) | Détection fin de parole | Implémenté |
| Collage curseur automatique | Sortie du pipeline | Implémenté |
| 3 modes d'écriture (Chat/Pro/Code) | Différenciation vs dictée basique | Implémenté |
| LLM conditionnel (Ollama) | Mode Pro/Smart Formatting | Implémenté |
| Menu bar app | UX macOS native | Implémenté |
| Overlay visuel (ondes simples) | Feedback pendant dictée | À redesigner |
| Thème "cahier de classe" | Identité DictAI | À implémenter |
| Son stylo sur papier | Feedback activation | À implémenter |
| Interface simplifiée (3 sections) | UX non-tech | À redesigner |
| Historique local (SQLite) | Récupération de texte | Existant (Handy) |

**Explicitement hors MVP :**
- Streaming transcription (chunks)
- Vocabulaire personnel
- Snippets
- Android / iOS
- Sync Supabase
- Auto-update

### Post-MVP Features

**Phase 2 — Growth (M+3 à M+6) :**

| Feature | Priorité | Dépendance |
|---------|----------|------------|
| Streaming transcription (chunks 1-2s) | P0 | Refactor pipeline async |
| Vocabulaire personnel | P1 | SQLite + UI section |
| Overlay scripturale (boucles cursives) | P1 | Canvas/SVG animation |
| Auto-update (Tauri Updater) | P1 | GitHub Releases CI/CD |
| Snippets | P2 | SQLite + déclencheurs vocaux |
| Code signing + Notarization | P0 | Certificat dev Apple |

**Phase 3 — Expansion (M+6 à M+12) :**

| Feature | Priorité | Dépendance |
|---------|----------|------------|
| Android natif (Kotlin) | P0 | Nouveau projet, pipeline Whisper Android |
| Sync Supabase (paywall 4,99 EUR) | P0 | Backend Supabase, auth, API |
| Buy Me a Coffee intégration | P2 | Lien simple |
| Fine-tuning Whisper FR (QLoRA) | P1 | Dataset FR, notebook Colab |
| Détection de contexte (app active) | P2 | macOS API NSWorkspace |

**Phase 4 — Vision (M+12+) :**
- iOS natif (Swift)
- Multi-langues
- API/Plugin (VS Code, Obsidian)
- Communauté de règles
- Windows/Linux

### Risk Mitigation Strategy

**Technical Risks :**
- *Plus gros défi technique :* Streaming transcription par chunks sans perte de cohérence inter-chunks. Mitigation : commencer par des chunks indépendants (phrase par phrase), puis ajouter le contexte inter-chunks en V2.
- *Hypothèse risquée :* Whisper Q5 local atteint une qualité FR suffisante pour un usage quotidien. Mitigation : benchmark WER sur corpus FR dès le MVP, fallback cloud opt-in si WER > 15%.

**Market Risks :**
- *Risque principal :* Le marché francophone de la dictée intelligente est trop niche. Mitigation : open-source pour maximiser la visibilité, communauté GitHub comme signal de traction.
- *Validation :* Lancement beta sur r/france, r/devfr, forums productivité FR. 50 utilisateurs actifs en 1 mois = signal positif.

**Resource Risks :**
- *Solo dev :* Si moins de temps disponible, réduire le MVP au pipeline core (raccourci → Whisper → rules → collage) sans redesign UI. L'interface Handy existante fonctionne, même si elle n'est pas optimale.
- *Feature minimum absolue :* Un .dmg qui s'installe, un raccourci qui transcrit et colle du texte propre en français. Tout le reste est bonus.

## Functional Requirements

### Dictation Pipeline

- **FR1:** L'utilisateur peut démarrer une dictée via un raccourci clavier global configurable, depuis n'importe quelle application
- **FR2:** L'utilisateur peut arrêter la dictée en relâchant le raccourci ou via un second appui
- **FR3:** Le système peut capturer l'audio du microphone et le transcrire en texte français via Whisper
- **FR4:** Le système peut détecter automatiquement le début et la fin de parole (VAD)
- **FR5:** Le système peut appliquer des règles de nettoyage FR au texte transcrit (filler words, élisions, ponctuation doublée, bégaiements, espacement)
- **FR6:** Le système peut router conditionnellement le texte vers le LLM local selon le score de confiance, le nombre de mots et le mode d'écriture
- **FR7:** Le système peut coller automatiquement le texte traité au curseur actif dans l'application au premier plan

### Writing Modes

- **FR8:** L'utilisateur peut sélectionner un mode d'écriture parmi Chat, Pro et Code
- **FR9:** En mode Chat, le système peut appliquer une correction orthographique minimale et une ponctuation basique tout en conservant le ton original
- **FR10:** En mode Pro, le système peut reformuler le texte de manière concise et professionnelle via le LLM local
- **FR11:** En mode Code, le système peut préserver le jargon technique et formater en Markdown

### Audio & Feedback

- **FR12:** Le système peut jouer un son de confirmation à l'activation de la dictée
- **FR13:** Le système peut afficher un overlay visuel pendant la dictée indiquant que l'enregistrement est actif
- **FR14:** L'utilisateur peut voir l'état de la dictée (inactive, écoute, traitement) via l'overlay

### History

- **FR15:** Le système peut enregistrer chaque session de dictée dans un historique local (texte brut, texte traité, horodatage, mode)
- **FR16:** L'utilisateur peut consulter l'historique des dictées passées
- **FR17:** L'utilisateur peut copier un texte depuis l'historique

### Settings & Configuration

- **FR18:** L'utilisateur peut configurer le raccourci clavier global
- **FR19:** L'utilisateur peut choisir le microphone source
- **FR20:** L'utilisateur peut activer/désactiver le post-traitement LLM
- **FR21:** L'utilisateur peut activer/désactiver le mode debug
- **FR22:** L'utilisateur peut activer le lancement automatique au démarrage macOS
- **FR23:** L'utilisateur peut accéder aux paramètres via l'icône menu bar

### User Interface

- **FR24:** L'utilisateur peut naviguer entre les sections de l'interface (Accueil, Style, Paramètres)
- **FR25:** L'utilisateur peut voir les informations de l'application (version, licence, crédits)
- **FR26:** L'interface peut afficher le thème "cahier de classe" (palette papier crème / encre verte)

### Privacy & Permissions

- **FR27:** Le système peut fonctionner entièrement hors ligne sans aucune connexion réseau
- **FR28:** Le système peut demander les permissions macOS nécessaires (Microphone, Accessibility) avec des explications en français
- **FR29:** L'utilisateur peut vérifier qu'aucune donnée ne quitte sa machine (indicateur de statut réseau)

### Debug & Monitoring

- **FR30:** En mode debug, l'utilisateur peut voir les métriques du pipeline (temps VAD, temps Whisper, confiance, route, temps rules/LLM, temps total)
- **FR31:** En mode debug, l'utilisateur peut voir le texte brut Whisper avant nettoyage
- **FR32:** En mode debug, l'utilisateur peut voir la route choisie (rules-only vs rules+LLM) et la raison

### Model Management

- **FR33:** Le système peut détecter la présence du modèle Whisper au démarrage
- **FR34:** Le système peut guider l'utilisateur pour le téléchargement initial du modèle si absent
- **FR35:** Le système peut détecter la présence d'Ollama et informer l'utilisateur si absent (fallback rules-only)

## Non-Functional Requirements

### Performance

- **NFR1:** La latence pipeline complète (fin de parole → texte collé) doit être < 3s au p95 pour une phrase de 15 mots en mode rules-only
- **NFR2:** La latence pipeline avec LLM (mode Pro) doit être < 5s au p95 pour une phrase de 15 mots
- **NFR3:** Le temps d'exécution des règles FR doit rester < 1ms au p99 (actuellement ~415 µs)
- **NFR4:** Le temps de détection VAD doit rester < 1ms au p99 (actuellement ~170 µs)
- **NFR5:** L'application doit consommer < 4 GB de RAM en fonctionnement (Whisper + LLM chargés)
- **NFR6:** Le démarrage de l'application (cold start) doit être < 10s incluant le chargement du modèle Whisper
- **NFR7:** L'application doit avoir un CPU idle < 1% quand aucune dictée n'est active

### Security & Privacy

- **NFR8:** Aucune donnée audio ou textuelle ne doit quitter la machine sans consentement explicite de l'utilisateur
- **NFR9:** L'historique local (SQLite) doit être stocké dans le répertoire Application Support de l'utilisateur avec les permissions fichier standard macOS
- **NFR10:** Les analytics opt-in (si activées) ne doivent contenir aucune donnée personnelle, audio ou textuelle — uniquement des compteurs d'usage anonymisés
- **NFR11:** Aucun credential, clé API ou token ne doit être stocké en dur dans le code source

### Reliability

- **NFR12:** Le crash rate doit être < 1% des sessions de dictée pour le MVP, < 0.1% à 12 mois
- **NFR13:** En cas d'échec du LLM (Ollama indisponible), le système doit basculer silencieusement en mode rules-only sans perte de la dictée en cours
- **NFR14:** En cas d'erreur de collage curseur, le texte traité doit rester disponible dans le presse-papier et l'historique
- **NFR15:** L'application doit survivre aux mises en veille / réveil macOS sans perdre sa configuration

### Accessibility

- **NFR16:** L'interface settings doit être navigable entièrement au clavier
- **NFR17:** L'interface doit être compatible VoiceOver (labels, rôles ARIA) au niveau basique
- **NFR18:** Le contraste texte/fond doit respecter WCAG 2.1 AA (ratio minimum 4.5:1)

### Compatibility

- **NFR19:** L'application doit fonctionner sur macOS 13 (Ventura) et versions ultérieures
- **NFR20:** L'application doit fonctionner sur Apple Silicon (M1+) et Intel (avec performance dégradée acceptée)
- **NFR21:** Le collage curseur doit fonctionner dans les applications standards (navigateurs, éditeurs de texte, suites bureautiques, terminaux)
- **NFR22:** L'application doit coexister avec d'autres outils utilisant des raccourcis clavier globaux (détection de conflits)

### Localization

- **NFR23:** L'interface utilisateur doit être entièrement en français
- **NFR24:** Les messages d'erreur et d'onboarding doivent être en français avec un ton accessible (pas de jargon technique)
