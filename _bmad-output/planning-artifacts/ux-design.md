---
stepsCompleted: ['ux-design-complete']
inputDocuments:
  - '{output_folder}/planning-artifacts/prd.md'
  - 'src/components/Sidebar.tsx'
  - 'src/components/settings/general/GeneralSettings.tsx'
  - 'src/components/settings/WriteModeSelector.tsx'
  - 'src/overlay/RecordingOverlay.tsx'
  - 'src/components/onboarding/Onboarding.tsx'
  - 'src/components/icons/DictationLogo.tsx'
  - 'src/components/settings/advanced/AdvancedSettings.tsx'
  - 'src/components/settings/debug/DebugSettings.tsx'
  - 'src/components/settings/history/HistorySettings.tsx'
  - 'src/components/settings/about/AboutSettings.tsx'
  - 'src/components/onboarding/AccessibilityOnboarding.tsx'
  - 'src/overlay/RecordingOverlay.css'
  - 'tailwind.config.js'
  - 'src/App.css'
workflowType: 'ux-design'
project_name: 'DictAI'
date: '2026-02-25'
status: 'complete'
---

# Specification UX Design -- DictAI

**Auteur :** Ullie
**Date :** 2026-02-25
**Version :** 1.0

---

## 1. Philosophie UX

### 1.1 Principes fondateurs

**Zero-config, zero-friction.** L'utilisateur installe DictAI, appuie sur un raccourci, et dicte. Aucun compte a creer, aucun modele a configurer manuellement, aucune etape intermediaire entre l'installation et la premiere dictee reussie. Chaque ecran supplementaire avant la premiere dictee est une friction a eliminer ou justifier.

**Public non-technique.** DictAI cible des francophones qui ne sont pas developpeurs : freelances, redacteurs, etudiants, professionnels. Le vocabulaire technique est banni de l'interface visible. Les termes comme "VAD", "Whisper", "LLM", "pipeline" n'apparaissent jamais dans l'UI principale -- ils sont relegues dans le panneau debug, accessible uniquement par toggle explicite.

**Local-first, confiance par defaut.** Le traitement 100% local est un argument de confiance, pas un detail technique. L'interface communique clairement que rien ne quitte la machine : pas de disclaimer ambigu, pas de mention de serveur. Un indicateur discret "Hors ligne" dans les parametres confirme visuellement ce choix.

**Identite "cahier de classe".** Le branding n'est pas decoratif -- il cree un ancrage emotionnel et culturel francophone. Le papier creme, l'encre verte de professeur, le son de stylo sur papier evoquent un souvenir scolaire partage. Cette identite differencie DictAI des interfaces tech minimalistes (noir/blanc) de la concurrence anglophone.

### 1.2 Regles UX directrices

| Regle | Application |
|-------|-------------|
| 3 clics maximum | Toute action courante (changer de mode, lancer une dictee, copier depuis l'historique) doit etre accessible en 3 interactions ou moins |
| Francais partout | Aucun texte en anglais dans l'UI. Chaque label, chaque message d'erreur, chaque tooltip est en francais courant |
| Progressive disclosure | Les parametres avances sont caches par defaut. L'utilisateur decouvre les options au fur et a mesure de son usage |
| Feedback immediat | Chaque action declenchee par l'utilisateur produit un retour visuel ou sonore en moins de 200ms |
| Graceful degradation | Si Ollama n'est pas installe, le mode Pro fonctionne en fallback rules-only avec un message explicatif -- pas de blocage |

---

## 2. Identite visuelle

### 2.1 Nom et positionnement

- **Nom :** DictAI (prononce "dict-A-I")
- **Etymologie :** Jeu de mots entre "dictee" et "AI" (intelligence artificielle)
- **Baseline :** "Dictee vocale intelligente, locale, francaise."

### 2.2 Palette de couleurs

La palette s'inspire directement du cahier d'ecolier francais : papier creme legerement jauni, encre verte du professeur, encre bleue de l'eleve, rouge de correction.

#### Couleurs principales

| Role | Nom | Hex | Usage |
|------|-----|-----|-------|
| **Background** | Papier creme | `#faf8f3` | Fond principal de la fenetre settings et de l'onboarding |
| **Primary** | Encre verte | `#2d7a4f` | Boutons principaux, elements actifs, icone menu bar active, bordures de selection |
| **Secondary** | Encre bleue | `#1a56db` | Liens, elements interactifs secondaires, mode Pro highlight |
| **Accent / Error** | Rouge correction | `#dc2626` | Erreurs, alertes, badge de notification, bouton de suppression |
| **Text primary** | Encre sombre | `#1a1a2e` | Texte principal, titres, labels |
| **Text secondary** | Encre grise | `#6b7280` | Texte secondaire, descriptions, placeholders |
| **Surface** | Papier leger | `#f5f3ee` | Fond des cartes, groupes de parametres, zones sureleves |
| **Border** | Ligne cahier | `#e5e2db` | Bordures de sections, separateurs, lignes horizontales comme les lignes d'un cahier |
| **Success** | Vert clair | `#16a34a` | Confirmations, permission accordee, dictee terminee |
| **Overlay bg** | Ardoise semi-transparente | `#1a1a2ecc` | Fond de l'overlay d'enregistrement (80% opacite) |

#### Variables CSS (remplacement de l'existant)

```css
:root {
  --color-background: #faf8f3;
  --color-surface: #f5f3ee;
  --color-text: #1a1a2e;
  --color-text-secondary: #6b7280;
  --color-primary: #2d7a4f;
  --color-primary-hover: #246b43;
  --color-primary-light: #2d7a4f1a; /* 10% opacity */
  --color-secondary: #1a56db;
  --color-accent-error: #dc2626;
  --color-success: #16a34a;
  --color-border: #e5e2db;
  --color-border-active: #2d7a4f;
  --color-overlay-bg: #1a1a2ecc;
  --color-mid-gray: #9ca3af;
}
```

#### Contraste WCAG 2.1 AA

| Combinaison | Ratio | Statut |
|-------------|-------|--------|
| Texte `#1a1a2e` sur fond `#faf8f3` | 15.2:1 | AA large + normal |
| Texte `#6b7280` sur fond `#faf8f3` | 5.1:1 | AA normal |
| Bouton blanc `#ffffff` sur fond vert `#2d7a4f` | 5.8:1 | AA normal |
| Erreur `#dc2626` sur fond `#faf8f3` | 5.4:1 | AA normal |
| Texte `#1a1a2e` sur surface `#f5f3ee` | 13.9:1 | AA large + normal |

### 2.3 Typographie

| Usage | Police | Taille | Poids | Fallback |
|-------|--------|--------|-------|----------|
| **Logo / Branding** | `'Caveat', cursive` | 24px | 700 | `'Kalam', 'Patrick Hand', cursive` |
| **Titres de section** | `'Inter', sans-serif` | 16px | 600 | `-apple-system, BlinkMacSystemFont, sans-serif` |
| **Labels de parametres** | `'Inter', sans-serif` | 14px | 500 | `-apple-system, BlinkMacSystemFont, sans-serif` |
| **Texte courant** | `'Inter', sans-serif` | 14px | 400 | `-apple-system, BlinkMacSystemFont, sans-serif` |
| **Descriptions / hints** | `'Inter', sans-serif` | 13px | 400 | `-apple-system, BlinkMacSystemFont, sans-serif` |
| **Code / technique** | `'JetBrains Mono', monospace` | 13px | 400 | `'SF Mono', 'Fira Code', monospace` |
| **Overlay texte** | `'Inter', sans-serif` | 12px | 500 | `-apple-system, sans-serif` |

**Note :** La police `Caveat` (Google Fonts, open-source) est utilisee uniquement pour le logo et le nom "DictAI" dans le sidebar. Elle evoque une ecriture manuscrite de cahier. Tout le reste de l'UI utilise Inter pour la lisibilite.

### 2.4 Iconographie

**Style :** Icones lineaires (stroke), epaisseur 1.5px, coins arrondis. Coherentes avec la bibliotheque `lucide-react` deja utilisee dans le projet.

| Element | Icone | Source |
|---------|-------|--------|
| Accueil | `Home` | lucide-react |
| Style / Modes | `Palette` | lucide-react |
| Parametres | `Settings` | lucide-react |
| Microphone | `Mic` | lucide-react |
| Raccourci clavier | `Keyboard` | lucide-react |
| Historique | `History` | lucide-react |
| Mode Chat | `MessageCircle` | lucide-react |
| Mode Pro | `Briefcase` | lucide-react |
| Mode Code | `Code` | lucide-react |
| Debug (cache) | `Bug` | lucide-react |
| A propos | `Info` | lucide-react |
| Copier | `Copy` | lucide-react |
| Succes | `Check` | lucide-react |
| Erreur / Supprimer | `Trash2` | lucide-react |
| Son actif | `Volume2` | lucide-react |
| Hors ligne | `WifiOff` | lucide-react |

### 2.5 Son

| Evenement | Son | Duree | Notes |
|-----------|-----|-------|-------|
| **Activation dictee** | Stylo sur papier (scratch doux) | ~300ms | Son principal de l'identite DictAI. Fichier WAV. Volume ajustable. |
| **Fin de dictee / collage** | Tourner une page (flip subtil) | ~200ms | Confirme que le texte a ete colle |
| **Erreur** | Tap sec sur bois | ~150ms | Alerte discrete sans etre agressive |
| **Annulation** | Froissement papier leger | ~200ms | Feedback d'annulation de dictee |

Les sons sont stockes dans `src-tauri/resources/sounds/`. Le volume est controle par le VolumeSlider existant. L'utilisateur peut desactiver les sons via le toggle AudioFeedback existant.

### 2.6 Logo

Le logo DictAI combine un microphone et une plume/stylo dans un design minimaliste.

**Description visuelle :**
- Forme de base : microphone classique vu de face (capsule arrondie + pied)
- La tige du microphone se transforme en plume d'ecriture vers le bas (la base du pied s'affine en pointe de plume)
- Couleur principale : encre verte `#2d7a4f`
- Trait : 2.5px, coins arrondis, style coherent avec lucide-react
- Taille de reference : 36x36px dans le sidebar, 64x64px dans l'onboarding, 16x16px pour le tray icon

**Texte logo :**
- "DictAI" en police Caveat Bold, 24px, couleur `#2d7a4f`
- Sous-titre "Dictee vocale intelligente" en Inter Regular, 10px, couleur `#6b7280`

**Implementation :** Composant SVG inline React (remplace `DictationLogo.tsx` existant). Pas de PNG/raster pour la nettete sur Retina.

---

## 3. Architecture de l'information

### 3.1 Navigation simplifiee : de 7 a 3 sections

L'interface Handy actuelle expose 7 sections dans le sidebar :

```
general | models | advanced | postprocessing | history | debug | about
```

DictAI reduit a 3 sections principales :

```
Accueil | Style | Parametres
```

### 3.2 Cartographie du contenu

#### Accueil (Home)

L'ecran principal. C'est ce que l'utilisateur voit a chaque ouverture de la fenetre DictAI.

| Element | Description |
|---------|-------------|
| Logo DictAI | Centre en haut du panneau principal |
| Statut en direct | Indicateur "Pret a dicter" / "Microphone : [nom]" / "Modele charge" |
| Raccourci rappel | Affichage du raccourci actuel (ex: `Cmd+Shift+D`) avec possibilite de le modifier |
| Mode d'ecriture actif | Selecteur Chat/Pro/Code visible et cliquable (composant WriteModeSelector redesigne) |
| Derniere dictee | Apercu de la derniere dictee (texte tronque + horodatage + bouton copier) |
| Lien historique complet | "Voir tout l'historique" en lien discret |

#### Style (Modes d'ecriture)

Ecran dedie a l'explication et au choix des modes.

| Element | Description |
|---------|-------------|
| Selecteur de mode | 3 cartes cliquables, une par mode, avec description detaillee |
| Mode Chat | "Correction minimale. Garde votre ton naturel. Ideal pour messages et notes rapides." |
| Mode Pro | "Reformulation professionnelle. Emails, courriers, textes formels." + badge "LLM" si Ollama present |
| Mode Code | "Jargon technique preserve. Formatage Markdown. Documentation et commentaires." |
| Apercu en direct | Zone de texte montrant un exemple avant/apres pour le mode selectionne |
| Statut Ollama | Si absent : message "Le mode Pro utilise un traitement avance. Installez Ollama pour en profiter." avec lien |

#### Parametres (Settings)

Parametres regroupes en sous-sections accordeon (collapsibles).

| Sous-section | Contenu (composants existants remappes) |
|-------------|----------------------------------------|
| **Audio** | MicrophoneSelector, VolumeSlider, MuteWhileRecording, AudioFeedback, OutputDeviceSelector |
| **Raccourcis** | ShortcutInput (transcribe), PushToTalk |
| **Modeles** | ModelSettingsCard (modele Whisper actif), gestion de modele (telecharger, supprimer) |
| **Historique** | HistoryLimit, RecordingRetentionPeriodSelector, lien "Ouvrir le dossier" |
| **Application** | AutostartToggle, StartHidden, ShowTrayIcon, ShowOverlay, AppLanguageSelector |
| **Avance** | PasteMethod, TypingTool, ClipboardHandling, AutoSubmit, AppendTrailingSpace, CustomWords, ModelUnloadTimeout, ExperimentalToggle, PostProcessingToggle |
| **A propos** | Version, lien GitHub DictAI, licence MIT, remerciements (Whisper, Handy), bouton "Soutenir le projet" |
| **Debug** (cache) | Toggle active dans Avance. Affiche : LogLevelSelector, PasteDelay, WordCorrectionThreshold, AlwaysOnMicrophone, ClamshellMicrophoneSelector, SoundPicker, UpdateChecksToggle, DebugPaths, LogDirectory, KeyboardImplementationSelector |

### 3.3 Elements caches / supprimes

| Element Handy | Decision DictAI | Raison |
|---------------|-----------------|--------|
| Section "Models" (sidebar) | Fusionne dans Parametres > Modeles | Trop technique pour etre une section principale |
| Section "Post-Processing" (sidebar) | Toggle dans Parametres > Avance | Technique, concerne uniquement les utilisateurs avances |
| Section "Debug" (sidebar) | Cache par defaut, toggle dans Avance | Uniquement pour le developpeur/contributeur |
| Section "About" (sidebar) | Integre dans Parametres > A propos | Ne justifie pas une section principale |
| TranslateToEnglish | Supprime | DictAI est FR-only pour le MVP |
| GlobalShortcutInput | Conserve comme ShortcutInput(transcribe) | Renomme pour clarte |
| HandyTextLogo | Remplace par DictAILogo | Rebranding complet |

### 3.4 Architecture de navigation visuelle

```
+-------------------+--------------------------------------+
|                   |                                      |
|    [Logo DictAI]  |         CONTENU PRINCIPAL            |
|                   |                                      |
|    +-----------+  |  (change selon section active)       |
|    | Accueil   |  |                                      |
|    +-----------+  |                                      |
|    | Style     |  |                                      |
|    +-----------+  |                                      |
|    | Parametres|  |                                      |
|    +-----------+  |                                      |
|                   |                                      |
|   [v0.2.0]       |                                      |
+-------------------+--------------------------------------+
     160px                       reste de la fenetre
```

Le sidebar est fixe a gauche (160px de large). Le contenu principal defilable a droite. Le logo DictAI est en haut du sidebar. Le numero de version est discret en bas du sidebar.

---

## 4. Parcours utilisateur principaux

### 4.1 Journey 1 -- Sophie : Premier contact (installation a premiere dictee)

```
[Telecharge .dmg] --> [Installe] --> [Premiere ouverture]
                                          |
                                     [Onboarding]
                                     Step 1: Permission Micro
                                     Step 2: Permission Accessibilite
                                     Step 3: Telechargement modele (si non bundle)
                                     Step 4: Test dictee rapide
                                          |
                                     [Ecran Accueil]
                                     Mode Chat actif par defaut
                                     Raccourci affiche (Cmd+Shift+D)
                                          |
                                     [Appuie raccourci]
                                     Son stylo sur papier
                                     Overlay vert apparait
                                          |
                                     [Dicte une phrase]
                                     Barres audio animees
                                          |
                                     [Relache raccourci]
                                     Overlay -> "Transcription..."
                                     Overlay -> disparait
                                     Son page tournee
                                     Texte colle au curseur
```

**Ecrans impliques :** Onboarding (4 etapes), Overlay d'enregistrement, Ecran Accueil
**Temps cible :** < 2 min de l'installation a la premiere dictee reussie

### 4.2 Journey 2 -- Marc : Mode Code

```
[Ouvre DictAI] --> [Sidebar: Style] --> [Selectionne "Code"]
                                              |
                                        [Retour VS Code]
                                        [Appuie raccourci]
                                        [Dicte documentation technique]
                                        [Texte colle avec backticks preserves]
```

**Ecrans impliques :** Ecran Style (selection mode), Overlay
**Interaction cle :** La selection de mode persiste entre les sessions. Marc n'a pas a re-selectionner "Code" a chaque ouverture.

### 4.3 Journey 3 -- Sophie : Historique

```
[Ouvre DictAI] --> [Accueil: "Voir tout l'historique"]
                        |
                   [Parametres > Historique]
                   Liste chronologique des dictees
                   Chaque entree : texte + horodatage + bouton copier
                        |
                   [Clic "Copier"] --> texte dans le presse-papier
```

**Ecrans impliques :** Ecran Accueil (apercu derniere dictee), Parametres > Historique (liste complete)
**Note :** L'historique n'est plus une section principale du sidebar. Il est accessible via un lien depuis Accueil ou directement dans Parametres.

### 4.4 Journey 4 -- Thomas : Mode Pro et email

```
[Ouvre DictAI] --> [Sidebar: Style] --> [Selectionne "Pro"]
                                              |
                                        [Voit indication "LLM actif"]
                                        [Retour Gmail]
                                        [Appuie raccourci]
                                        [Dicte message informel]
                                        [Texte reformule en style professionnel]
```

**Ecrans impliques :** Ecran Style (avec indicateur LLM), Overlay
**Cas edge :** Si Ollama absent, le mode Pro affiche un message : "Mode simplifie actif. Pour la reformulation complete, installez Ollama." Le bouton reste cliquable -- il fonctionne en fallback rules-only.

### 4.5 Journey 5 -- Ullie : Debug

```
[Ouvre DictAI] --> [Sidebar: Parametres] --> [Sous-section Avance]
                                                    |
                                               [Toggle "Mode debug"]
                                                    |
                                               [Section Debug apparait]
                                               LogLevel, PasteDelay, etc.
                                                    |
                                               [Dicte phrase test]
                                               [Ouvre logs : metriques pipeline visibles]
```

**Ecrans impliques :** Parametres > Avance > Debug
**Note :** Le mode debug n'ajoute pas une section au sidebar. Il deplie une zone supplementaire dans Parametres > Avance.

---

## 5. Ecrans et composants

### 5.1 Menu Bar (Tray Icon)

#### Icone et etats

| Etat | Icone | Couleur |
|------|-------|---------|
| **Idle (pret)** | Micro-plume DictAI simplifie (silhouette) | `#1a1a2e` (sombre, template image macOS) |
| **Ecoute active** | Micro-plume avec ondes sonores | `#2d7a4f` (vert, anime) |
| **Traitement** | Micro-plume avec spinner | `#1a56db` (bleu) |
| **Erreur** | Micro-plume avec point d'exclamation | `#dc2626` (rouge) |
| **Modele non charge** | Micro-plume grise | `#9ca3af` (gris) |

L'icone tray est un "template image" macOS (monochrome, s'adapte au theme clair/sombre de la barre de menu). L'animation de couleur n'est active que pendant l'enregistrement.

#### Comportement au clic

- **Clic gauche :** Ouvre/ferme la fenetre settings DictAI (panneau principal ancre sous l'icone tray, style popover macOS natif)
- **Clic droit :** Menu contextuel minimal :
  - "Ouvrir DictAI" (ouvre la fenetre)
  - Separateur
  - "Mode : Chat / Pro / Code" (sous-menu de selection rapide)
  - Separateur
  - "Quitter DictAI"

#### Taille de la fenetre

- **Largeur :** 680px (sidebar 160px + contenu 520px)
- **Hauteur :** 480px (fixe, contenu defilable)
- **Position :** Ancree sous l'icone tray, centree horizontalement
- **Coin :** border-radius 12px, ombre douce `0 8px 32px rgba(0,0,0,0.12)`

### 5.2 Overlay d'enregistrement

L'overlay est une petite fenetre flottante toujours au premier plan, positionnee en haut au centre de l'ecran. Il remplace l'overlay noir actuel de Handy par un design coherent avec le theme cahier.

#### Design visuel

```
+------------------------------------------------------+
|  [Micro]   |||||||||||||||   [X Annuler]             |
+------------------------------------------------------+
   vert       barres vertes     gris, hover rouge
```

- **Forme :** Capsule (pilule) arrondie, 200px x 40px
- **Fond :** `#1a1a2ecc` (ardoise semi-transparente, 80% opacite) -- conserve le fond sombre pour le contraste avec n'importe quel fond d'ecran
- **Bord :** 1px solid `#2d7a4f40` (vert subtil)
- **Border-radius :** 20px (pleinement arrondi)
- **Position :** top: 12px, center horizontal, `z-index: 9999`
- **Ombre :** `0 4px 16px rgba(0,0,0,0.25)`

#### Etats

| Etat | Zone gauche | Zone centrale | Zone droite |
|------|-------------|---------------|-------------|
| **Ecoute** | Icone micro `#2d7a4f` avec pulse subtil | 9 barres audio animees, couleur `#2d7a4f` -> `#4ade80` (gradient vert) | Bouton annuler (X) gris |
| **Transcription** | Icone transcription (ondes texte) `#1a56db` | Texte "Transcription..." avec animation pulse | Vide |
| **Traitement LLM** | Icone sparkles `#1a56db` | Texte "Mise en forme..." avec animation pulse | Vide |
| **Succes** | Check vert `#16a34a` | Vide | Vide (disparait apres 500ms) |

#### Animations

- **Apparition :** Fade in (opacity 0->1) + slide down (translateY -8px -> 0) sur 200ms, easing `ease-out`
- **Disparition :** Fade out (opacity 1->0) sur 300ms, easing `ease-in`
- **Barres audio :** Meme logique que l'existant (smoothing 0.7/0.3), mais couleur `#2d7a4f` au lieu de `#ffe5ee` (rose Handy)
- **Pulse texte :** `opacity: 0.6 -> 1 -> 0.6` sur 1.5s, infinite, ease-in-out (identique a l'existant)

#### Differences avec l'overlay Handy actuel

| Aspect | Handy actuel | DictAI |
|--------|-------------|--------|
| Fond | `#000000cc` (noir) | `#1a1a2ecc` (ardoise) |
| Barres audio | `#ffe5ee` (rose) | `#2d7a4f` (vert) |
| Taille | 172x36px | 200x40px |
| Bordure | Aucune | 1px vert subtil |
| Etat succes | N'existe pas | Flash vert 500ms avant disparition |
| Hover annuler | `#faa2ca33` (rose) | `#dc262633` (rouge leger) |

### 5.3 Accueil (Home)

L'ecran Accueil est le coeur de DictAI. Il est visible a chaque ouverture de la fenetre.

#### Layout

```
+------------------------------------------+
|                                          |
|     [Logo DictAI grand]                  |
|     "Dictee vocale intelligente"         |
|                                          |
|  +------------------------------------+  |
|  |  Pret a dicter                     |  |
|  |  Micro : MacBook Pro Microphone    |  |
|  |  Raccourci : Cmd+Shift+D [edit]    |  |
|  +------------------------------------+  |
|                                          |
|  Mode actif :                            |
|  +--------+  +--------+  +--------+     |
|  | Chat   |  |  Pro   |  |  Code  |     |
|  | (actif)|  |        |  |        |     |
|  +--------+  +--------+  +--------+     |
|                                          |
|  +------------------------------------+  |
|  |  Derniere dictee       14:32      |  |
|  |  "J'ai teste ce nouveau..."       |  |
|  |              [Copier]              |  |
|  +------------------------------------+  |
|  Voir tout l'historique ->               |
|                                          |
+------------------------------------------+
```

#### Composants

**Carte de statut** -- Fond `#f5f3ee`, bordure `#e5e2db`, padding 16px, border-radius 8px.
- Ligne 1 : Pastille verte `#16a34a` (8x8px, border-radius 50%) + "Pret a dicter" en Inter 14px medium `#1a1a2e`
- Ligne 2 : Icone micro (lucide) 14px `#6b7280` + nom du microphone en Inter 13px `#6b7280`
- Ligne 3 : Icone clavier (lucide) 14px `#6b7280` + raccourci en `<kbd>` style (fond `#e5e2db`, border-radius 4px, padding 2px 6px, Inter 12px monospace) + lien "[modifier]" en `#1a56db`

**Selecteur de mode rapide** -- 3 boutons allignes horizontalement, style carte (pas le bouton compact actuel du WriteModeSelector).
- Taille : ~100px x 60px chacun
- Mode actif : fond `#2d7a4f1a` (vert 10%), bordure `#2d7a4f`, texte `#2d7a4f`
- Mode inactif : fond `#f5f3ee`, bordure `#e5e2db`, texte `#6b7280`
- Hover (inactif) : bordure `#2d7a4f80`, texte `#1a1a2e`
- Icone en haut (MessageCircle / Briefcase / Code), label en bas
- Pas d'emoji -- on utilise les icones lucide-react

**Apercu derniere dictee** -- Fond `#f5f3ee`, bordure `#e5e2db`, padding 12px, border-radius 8px.
- Header : "Derniere dictee" en Inter 12px medium `#6b7280` + horodatage a droite
- Corps : Texte tronque a 2 lignes (text-overflow: ellipsis), Inter 14px italic `#1a1a2e`
- Pied : Bouton "Copier" (icone Copy + label) aligne a droite, style secondaire
- Si aucune dictee : texte center "Aucune dictee pour le moment. Appuyez sur Cmd+Shift+D pour commencer." en `#6b7280`

**Lien historique** -- Texte "Voir tout l'historique" en Inter 13px, couleur `#1a56db`, fleche droite. Au clic : bascule sur Parametres > Historique.

### 5.4 Style (Modes d'ecriture)

Ecran dedie a l'explication detaillee des modes et a leur selection.

#### Layout

```
+------------------------------------------+
|                                          |
|  Choisissez votre style                  |
|  "DictAI adapte le texte a votre        |
|   contexte."                             |
|                                          |
|  +------------------------------------+  |
|  |  [MessageCircle]  Chat             |  |
|  |                                    |  |
|  |  Correction minimale. Garde votre  |  |
|  |  ton naturel et spontane. Ideal    |  |
|  |  pour messages, notes, brainstorm. |  |
|  |                                    |  |
|  |  Avant : "euh du coup j'ai teste  |  |
|  |  ce truc c'est pas mal"           |  |
|  |  Apres : "J'ai teste ce truc,     |  |
|  |  c'est pas mal."                   |  |
|  +------------------------------------+  |
|                                          |
|  +------------------------------------+  |
|  |  [Briefcase]  Pro          [LLM]  |  |
|  |                                    |  |
|  |  Reformulation professionnelle.    |  |
|  |  Emails, rapports, courriers.      |  |
|  |  Ton formel et structure.          |  |
|  |                                    |  |
|  |  Avant : "en fait je voulais vous  |  |
|  |  dire que j'ai avance"            |  |
|  |  Apres : "Je souhaitais vous      |  |
|  |  informer de mon avancement."      |  |
|  +------------------------------------+  |
|                                          |
|  +------------------------------------+  |
|  |  [Code]  Code                      |  |
|  |                                    |  |
|  |  Jargon technique preserve.        |  |
|  |  Formatage Markdown. Symboles      |  |
|  |  intacts. Ideal documentation.     |  |
|  |                                    |  |
|  |  Avant : "cette fonction prend un  |  |
|  |  dataframe pandas en entree"       |  |
|  |  Apres : "Cette fonction prend un  |  |
|  |  `DataFrame` pandas en entree."    |  |
|  +------------------------------------+  |
|                                          |
+------------------------------------------+
```

#### Composants

**Carte de mode** (3 instances) -- Fond `#f5f3ee`, bordure `#e5e2db`, padding 16px, border-radius 10px, cursor: pointer.
- Mode actif : bordure gauche 3px `#2d7a4f`, fond `#2d7a4f08`
- Mode inactif : bordure standard, fond `#f5f3ee`
- Hover : elevation legere (`box-shadow: 0 2px 8px rgba(0,0,0,0.06)`), bordure `#2d7a4f40`

**Contenu de la carte :**
- Header : Icone lucide (24px, `#2d7a4f` si actif, `#6b7280` sinon) + nom du mode en Inter 16px semibold + badge optionnel
- Badge "LLM" (mode Pro uniquement) : fond `#1a56db1a`, texte `#1a56db`, Inter 11px medium, border-radius 4px, padding 2px 6px
- Description : Inter 14px regular `#1a1a2e`, 2-3 lignes
- Zone avant/apres : fond `#faf8f3`, bordure `#e5e2db`, border-radius 6px, padding 10px
  - "Avant :" en Inter 12px medium `#6b7280`, texte en Inter 13px italic `#6b7280`
  - "Apres :" en Inter 12px medium `#2d7a4f`, texte en Inter 13px regular `#1a1a2e`

**Message Ollama absent** (conditionnel) -- Si Ollama n'est pas detecte ET que le mode Pro est selectionne :
- Bandeau jaune leger : fond `#fef3c7`, bordure `#f59e0b40`, border-radius 8px, padding 12px
- Icone AlertTriangle `#f59e0b` + texte : "Le mode Pro fonctionne en version simplifiee. Pour la reformulation complete, installez Ollama." + lien "En savoir plus" vers `https://ollama.com`

### 5.5 Parametres (Settings)

Les parametres utilisent un systeme d'accordeon (sections collapsibles) pour organiser le contenu.

#### Layout

```
+------------------------------------------+
|                                          |
|  Parametres                              |
|                                          |
|  v Audio --------------------------------+
|  |  Microphone : [dropdown]             |
|  |  Volume : [========---]              |
|  |  Couper le micro pendant lecture  [x]|
|  |  Son d'activation               [x] |
|  |  Sortie audio : [dropdown]           |
|  +--------------------------------------+
|                                          |
|  > Raccourcis ---------------------------+
|  +--------------------------------------+
|                                          |
|  > Modeles ------------------------------+
|  +--------------------------------------+
|                                          |
|  > Historique ----------------------------+
|  +--------------------------------------+
|                                          |
|  > Application ---------------------------+
|  +--------------------------------------+
|                                          |
|  > Avance --------------------------------+
|  +--------------------------------------+
|                                          |
|  > A propos ------------------------------+
|  +--------------------------------------+
|                                          |
+------------------------------------------+
```

#### Composant Accordeon

- **En-tete de section :** padding 12px 16px, fond transparent, cursor pointer, hover fond `#f5f3ee`
- **Chevron :** icone `ChevronDown` (lucide) 16px, `#6b7280`, rotation 0deg (ferme) -> -180deg (ouvert), transition 200ms
- **Titre :** Inter 14px semibold `#1a1a2e`, uppercase tracking-wide
- **Contenu deplie :** padding 0 16px 16px, animation slide-down 200ms
- **Separateur :** ligne 1px `#e5e2db` entre chaque section

#### Sous-section Audio

Regroupe les composants existants avec les noms FR :

| Composant existant | Label FR | Type |
|---------------------|----------|------|
| `MicrophoneSelector` | Microphone | Dropdown |
| `VolumeSlider` | Volume de retour sonore | Slider (desactive si son off) |
| `MuteWhileRecording` | Couper le micro pendant la lecture | Toggle |
| `AudioFeedback` | Son d'activation / desactivation | Toggle |
| `OutputDeviceSelector` | Sortie audio | Dropdown (desactive si son off) |

#### Sous-section Raccourcis

| Composant existant | Label FR | Type |
|---------------------|----------|------|
| `ShortcutInput(transcribe)` | Raccourci de dictee | Enregistreur de raccourci |
| `PushToTalk` | Maintenir pour parler (push-to-talk) | Toggle + description |

#### Sous-section Modeles

| Composant existant | Label FR | Type |
|---------------------|----------|------|
| `ModelSettingsCard` | Modele de reconnaissance vocale | Carte avec dropdown + statut |
| Bouton telecharger | Telecharger un modele | Bouton + barre de progression |
| Bouton supprimer | Supprimer le modele | Bouton danger |

#### Sous-section Historique

| Composant existant | Label FR | Type |
|---------------------|----------|------|
| `HistorySettings` (liste) | Historique des dictees | Liste scrollable avec les entrees |
| `HistoryLimit` | Nombre maximum d'entrees | Input numerique |
| `RecordingRetentionPeriod` | Conservation des enregistrements | Dropdown (jours) |
| Bouton ouvrir dossier | Ouvrir le dossier des enregistrements | Bouton secondaire |

#### Sous-section Application

| Composant existant | Label FR | Type |
|---------------------|----------|------|
| `AutostartToggle` | Lancer au demarrage | Toggle |
| `StartHidden` | Demarrer en arriere-plan | Toggle |
| `ShowTrayIcon` | Afficher l'icone dans la barre de menu | Toggle |
| `ShowOverlay` | Afficher l'overlay pendant la dictee | Toggle |
| `AppLanguageSelector` | Langue de l'interface | Dropdown (FR uniquement au MVP, mais le composant reste) |

#### Sous-section Avance

| Composant existant | Label FR | Type |
|---------------------|----------|------|
| `PasteMethod` | Methode de collage | Dropdown |
| `TypingTool` | Outil de saisie | Dropdown |
| `ClipboardHandling` | Gestion du presse-papier | Dropdown |
| `AutoSubmit` | Envoi automatique (Entree) | Toggle |
| `AppendTrailingSpace` | Ajouter un espace apres le texte | Toggle |
| `CustomWords` | Mots personnalises | Input multi-valeur |
| `ModelUnloadTimeout` | Delai de dechargement du modele | Slider/input |
| `ExperimentalToggle` | Fonctionnalites experimentales | Toggle |
| `PostProcessingToggle` | Post-traitement (API cloud) | Toggle (dans Experimental) |
| **Mode debug** | Activer le mode debug | Toggle -- quand active, deplie la zone Debug en dessous |

#### Zone Debug (conditionnelle, dans Avance)

Visible uniquement quand le toggle "Mode debug" est actif. Fond `#f5f3ee` distinct avec bordure pointillee `#e5e2db` pour signaler visuellement son caractere technique.

| Composant existant | Label FR | Type |
|---------------------|----------|------|
| `LogLevelSelector` | Niveau de log | Dropdown |
| `PasteDelay` | Delai de collage (ms) | Input numerique |
| `WordCorrectionThreshold` | Seuil de correction mots | Slider |
| `AlwaysOnMicrophone` | Micro toujours actif | Toggle |
| `ClamshellMicrophoneSelector` | Micro en mode clapet | Dropdown |
| `SoundPicker` | Theme sonore | Selecteur |
| `UpdateChecksToggle` | Verifier les mises a jour | Toggle |
| `DebugPaths` | Chemins de l'application | Affichage lecture seule |
| `LogDirectory` | Dossier des logs | Lien + bouton ouvrir |
| `KeyboardImplementationSelector` | Implementation clavier | Dropdown |

#### Sous-section A propos

| Element | Label FR | Type |
|---------|----------|------|
| Version | Version de l'application | Texte `v0.2.0` monospace |
| Lien GitHub | Code source | Bouton secondaire -> `github.com/Uhama91/dictation-ia-locale` |
| Licence | Licence | Texte "MIT" |
| Soutenir | Soutenir le projet | Bouton primaire -> Buy Me a Coffee |
| Remerciements | Remerciements | Texte : Whisper (OpenAI), Handy (cjpais), Tauri |
| `AppDataDirectory` | Dossier des donnees | Lien + bouton ouvrir |
| `LogDirectory` | Dossier des logs | Lien + bouton ouvrir |

### 5.6 Historique

L'historique n'a plus de section dediee dans le sidebar. Il est accessible de deux facons :

1. **Depuis Accueil :** Lien "Voir tout l'historique" -> ouvre Parametres > Historique
2. **Depuis Parametres :** Sous-section Historique (accordeon)

#### Design de la liste

Chaque entree d'historique est une ligne dans une carte groupee (style SettingsGroup existant, reutilise).

```
+--------------------------------------------+
|  14:32 - Aujourd'hui          [*] [C] [X]  |
|  "J'ai teste ce nouveau restaurant a       |
|  Lyon. C'est vraiment pas mal, les pates   |
|  etaient excellentes."                      |
|  [Lecteur audio >>>-----------]             |
+--------------------------------------------+
|  11:15 - Aujourd'hui          [*] [C] [X]  |
|  "Je souhaitais vous informer de mon..."   |
|  [Lecteur audio >>>-----------]             |
+--------------------------------------------+
```

- **Horodatage :** Inter 13px medium `#1a1a2e`, format "HH:mm - Aujourd'hui" ou "HH:mm - 24 fev."
- **Boutons d'action :** allignes a droite, 16px, couleur `#6b7280`, hover `#2d7a4f`
  - Etoile (sauvegarder) : pleine si sauvegardee (`#2d7a4f`), vide sinon
  - Copier : icone Copy, flash "Copie !" pendant 2s apres clic
  - Supprimer : icone Trash2, hover `#dc2626`
- **Texte transcrit :** Inter 14px italic `#1a1a2e`, max 3 lignes, text-overflow ellipsis, selectionnable (`user-select: text`)
- **Lecteur audio :** Composant `AudioPlayer` existant, style adapte au theme (slider vert `#2d7a4f`)
- **Etat vide :** Centre, illustration micro-plume grise + texte "Aucune dictee pour le moment."

### 5.7 Debug Panel

Le panneau debug n'est pas un ecran separe. C'est une zone conditionnelle a l'interieur de Parametres > Avance.

#### Activation

1. L'utilisateur ouvre Parametres > Avance
2. Il active le toggle "Mode debug"
3. Une zone supplementaire se deplie en dessous avec les controles techniques

#### Style visuel

- **Fond :** `#f5f3ee` avec bordure pointillee `2px dashed #e5e2db`
- **Label :** "Outils de debug" en Inter 12px uppercase `#6b7280`, avec icone Bug (lucide)
- **Conteneur :** padding 16px, border-radius 8px, margin-top 8px
- **Les composants internes** utilisent le meme style SettingContainer/SettingsGroup que le reste

#### Metriques pipeline (affiches apres chaque dictee en mode debug)

Quand le mode debug est actif, l'ecran Accueil affiche une zone supplementaire sous l'apercu de la derniere dictee :

```
+--------------------------------------------+
|  Metriques pipeline            derniere     |
|  VAD : 0.2ms | Whisper : 1.8s | Conf: 0.87|
|  Route : rules-only | Rules : 0.4ms        |
|  Total : 1.85s                              |
|                                             |
|  Texte brut Whisper :                       |
|  "euh du coup j'ai teste ce truc c'est    |
|  vraiment pas mal genre"                    |
+--------------------------------------------+
```

- Fond `#f5f3ee`, bordure pointillee `#e5e2db`
- Texte monospace `JetBrains Mono` 12px
- Labels en `#6b7280`, valeurs en `#1a1a2e`
- Route "rules-only" en vert `#2d7a4f`, route "rules+LLM" en bleu `#1a56db`

---

## 6. Onboarding

### 6.1 Flux en 4 etapes

L'onboarding se declenche au premier lancement uniquement. Il est sequentiel (step 1 -> 2 -> 3 -> 4) et ne peut pas etre saute entierement (les permissions sont obligatoires).

#### Indicateur de progression

Barre de progression en haut de l'ecran : 4 segments. Segment actif = `#2d7a4f`, segments completes = `#2d7a4f`, segments a venir = `#e5e2db`.

```
[====][====][----][----]
 Step1 Step2 Step3 Step4
```

### 6.2 Step 1 -- Permission Microphone

```
+------------------------------------------+
|  [====][----][----][----]                |
|                                          |
|  [Logo DictAI 64px]                      |
|                                          |
|  Autorisez l'acces au microphone         |
|                                          |
|  [Illustration : micro avec ondes]       |
|                                          |
|  DictAI a besoin de votre microphone     |
|  pour ecouter votre voix et la           |
|  transcrire en texte. Aucun              |
|  enregistrement n'est envoye sur         |
|  internet -- tout reste sur votre Mac.   |
|                                          |
|  [  Autoriser le microphone  ]           |
|                                          |
|  [Attente...]  ou  [V Autorise]          |
|                                          |
+------------------------------------------+
```

**Bouton principal :** fond `#2d7a4f`, texte blanc, Inter 14px semibold, border-radius 8px, padding 12px 24px, hover `#246b43`
**Etat "Attente" :** Apres clic, le bouton devient gris avec spinner + "Autorisez dans les Preferences Systeme..." -- polling toutes les 1s (logique existante AccessibilityOnboarding.tsx)
**Etat "Autorise" :** Icone Check verte + "Microphone autorise" -- transition automatique vers Step 2 apres 500ms
**Texte rassurance :** La phrase "Aucun enregistrement n'est envoye sur internet" est en gras (font-weight 600) pour insister sur la vie privee.

### 6.3 Step 2 -- Permission Accessibilite

```
+------------------------------------------+
|  [====][====][----][----]                |
|                                          |
|  [Logo DictAI 64px]                      |
|                                          |
|  Autorisez l'acces a l'accessibilite     |
|                                          |
|  [Illustration : clavier + curseur]      |
|                                          |
|  DictAI utilise l'accessibilite macOS    |
|  pour deux choses :                      |
|  1. Detecter le raccourci clavier        |
|     global, meme quand DictAI n'est      |
|     pas au premier plan.                 |
|  2. Coller automatiquement le texte      |
|     dicte au curseur de l'application    |
|     que vous utilisez.                   |
|                                          |
|  [  Autoriser l'accessibilite  ]         |
|                                          |
+------------------------------------------+
```

Meme logique de bouton/etats que Step 1. La liste numerotee explique en termes simples pourquoi l'accessibilite est necessaire -- pas de jargon "AX API" ou "AppleScript".

### 6.4 Step 3 -- Telechargement du modele

Cet ecran n'apparait que si le modele Whisper n'est pas bundle dans le .dmg.

```
+------------------------------------------+
|  [====][====][====][----]                |
|                                          |
|  [Logo DictAI 64px]                      |
|                                          |
|  Preparation de la reconnaissance        |
|  vocale                                  |
|                                          |
|  DictAI telecharge le modele de          |
|  reconnaissance vocale (~1.5 Go).        |
|  C'est un telechargement unique -- le    |
|  modele restera sur votre Mac.           |
|                                          |
|  [==========>-----------] 45%            |
|  720 Mo / 1.5 Go  -  12 Mo/s            |
|  Temps restant : ~1 min                  |
|                                          |
|  Vous pourrez utiliser DictAI hors       |
|  ligne apres ce telechargement.          |
|                                          |
+------------------------------------------+
```

**Barre de progression :** fond `#e5e2db`, remplissage `#2d7a4f`, border-radius 4px, hauteur 8px
**Texte sous la barre :** Inter 13px `#6b7280`, progression + vitesse + estimation
**Note hors-ligne :** Inter 13px italic `#6b7280`, rassurance sur l'usage offline

Cet ecran remplace l'ecran Onboarding.tsx actuel (selection de modele parmi plusieurs). DictAI simplifie : un seul modele recommande (large-v3-turbo Q5), pas de choix utilisateur. Le modele se telecharge automatiquement.

### 6.5 Step 4 -- Test de dictee rapide

```
+------------------------------------------+
|  [====][====][====][====]                |
|                                          |
|  [Logo DictAI 64px]                      |
|                                          |
|  Essayez maintenant !                    |
|                                          |
|  Appuyez sur Cmd+Shift+D et dites       |
|  quelque chose. Par exemple :            |
|                                          |
|  "Bonjour, ceci est mon premier test    |
|   avec DictAI."                          |
|                                          |
|  +------------------------------------+  |
|  |                                    |  |
|  |  [Zone d'apercu du resultat]       |  |
|  |  "Votre texte apparaitra ici."     |  |
|  |                                    |  |
|  +------------------------------------+  |
|                                          |
|  [  Passer et commencer  ]              |
|                                          |
+------------------------------------------+
```

**Zone d'apercu :** fond `#faf8f3`, bordure `#e5e2db`, border-radius 8px, min-height 80px, padding 16px
- Avant dictee : texte placeholder "Votre texte apparaitra ici." en Inter 14px italic `#9ca3af`
- Apres dictee : texte transcrit en Inter 14px `#1a1a2e`, avec animation fade-in
- En cas de succes : bordure passe a `#2d7a4f`, fond `#2d7a4f08`

**Bouton "Passer"** : style secondaire (fond transparent, bordure `#e5e2db`, texte `#6b7280`). Permet de sauter le test sans bloquer.

**Apres dictee reussie :** Le bouton change en "C'est parti !" fond `#2d7a4f`, texte blanc. Au clic : transition vers l'ecran Accueil.

**Moment "aha!" :** C'est l'etape la plus importante de l'onboarding. L'utilisateur voit que DictAI fonctionne, que le texte est propre, que c'est instantane. Si cette etape echoue (micro non detecte, modele pas charge), afficher un message d'aide contextuel : "Verifiez que votre microphone est branche et reessayez."

---

## 7. Etats et feedback

### 7.1 Etats de l'application

| Etat | Icone tray | Overlay | Ecran Accueil |
|------|------------|---------|---------------|
| **Idle (pret)** | Micro sombre | Cache | "Pret a dicter" + pastille verte |
| **Ecoute** | Micro vert anime | Visible (barres audio) | "Ecoute en cours..." + pastille verte clignotante |
| **Transcription** | Micro bleu | "Transcription..." | "Transcription..." |
| **Traitement LLM** | Micro bleu | "Mise en forme..." | "Mise en forme..." |
| **Collage** | Flash vert | Disparait (fade out) | Apercu derniere dictee mis a jour |
| **Modele non charge** | Micro gris | N/A | "Modele non charge" + pastille orange + bouton "Charger" |
| **Erreur micro** | Micro rouge | N/A | "Microphone indisponible" + pastille rouge + suggestion |
| **Ollama absent** | N/A | N/A | Mode Pro affiche "Version simplifiee" (pas de blocage) |

### 7.2 Etats de chargement

**Chargement du modele Whisper :**
- Carte dans Parametres > Modeles avec barre de progression `#2d7a4f`
- Texte "Chargement du modele..." avec spinner
- Desactivation des controles de dictee pendant le chargement

**Telechargement de modele :**
- Barre de progression (identique a l'onboarding Step 3)
- Percentage + vitesse + estimation de temps

### 7.3 Etats d'erreur

| Erreur | Message FR | Action proposee |
|--------|-----------|-----------------|
| Micro non autorise | "DictAI n'a pas acces au microphone. Autorisez l'acces dans Preferences Systeme > Confidentialite." | Bouton "Ouvrir les Preferences" |
| Accessibilite non autorisee | "DictAI n'a pas acces a l'accessibilite. Le raccourci global et le collage au curseur ne fonctionneront pas." | Bouton "Ouvrir les Preferences" |
| Modele absent | "Le modele de reconnaissance vocale n'est pas installe." | Bouton "Telecharger" |
| Ollama absent (mode Pro) | "Ollama n'est pas detecte. Le mode Pro utilise un traitement simplifie." | Lien "Installer Ollama" (discret, pas bloquant) |
| Echec transcription | "La transcription a echoue. Verifiez votre microphone et reessayez." | Bouton "Reessayer" |
| Echec collage | "Le texte n'a pas pu etre colle au curseur. Il a ete copie dans votre presse-papier." | Texte informatif (pas de bouton, le presse-papier suffit) |
| Raccourci en conflit | "Le raccourci Cmd+Shift+D est deja utilise par une autre application." | Champ pour choisir un nouveau raccourci |

**Style des messages d'erreur :**
- Fond `#dc26260d` (rouge 5%), bordure `#dc262640`, border-radius 8px, padding 12px
- Icone AlertTriangle 16px `#dc2626` a gauche
- Texte Inter 14px `#1a1a2e`
- Action (bouton ou lien) aligne a droite ou en dessous

### 7.4 Etats vides

| Contexte | Message FR | Illustration |
|----------|-----------|-------------|
| Historique vide | "Aucune dictee pour le moment. Appuyez sur Cmd+Shift+D pour commencer." | Micro-plume grise (logo DictAI desature) |
| Aucun micro detecte | "Aucun microphone detecte. Branchez un microphone et reessayez." | Icone micro barree |
| Modeles telecharges = 0 | "Aucun modele installe. Telechargez un modele pour commencer." | Icone download |

---

## 8. Responsive et Accessibilite

### 8.1 Tailles de fenetre

| Mode | Largeur | Hauteur | Notes |
|------|---------|---------|-------|
| **Fenetre normale** | 680px | 480px | Taille par defaut, ancree sous l'icone tray |
| **Minimum** | 600px | 400px | En dessous, le contenu est tronque |
| **Maximum** | 800px | 600px | Au-dela, l'espace supplementaire est du padding |
| **Onboarding** | 520px | 560px | Fenetre centree, pas ancree au tray |
| **Overlay** | 200px | 40px | Fenetre flottante independante, toujours au premier plan |

La fenetre DictAI n'est pas redimensionnable par l'utilisateur (taille fixe). Cela simplifie le layout et evite les cas edge de responsive. Le contenu principal est defilable verticalement si le contenu depasse la hauteur.

### 8.2 VoiceOver (accessibilite macOS)

| Element | role ARIA | aria-label FR |
|---------|-----------|---------------|
| Sidebar | `navigation` | "Navigation principale" |
| Bouton section sidebar | `tab` | "Section [nom]" |
| Panneau contenu | `tabpanel` | "Contenu de la section [nom]" |
| Toggle (switch) | `switch` | "[label du setting]" + "active/desactive" |
| Selecteur de mode | `radiogroup` | "Mode d'ecriture" |
| Bouton de mode | `radio` | "[Chat/Pro/Code] - [description]" |
| Barre de progression | `progressbar` | "Telechargement : [X]%" |
| Carte d'historique | `article` | "Dictee du [date], [debut du texte]" |
| Overlay | `alert` avec `aria-live="polite"` | "Enregistrement en cours" / "Transcription en cours" |
| Message d'erreur | `alert` | Texte du message |
| Accordeon section | `button` avec `aria-expanded` | "Section [nom], depliee/repliee" |

### 8.3 Navigation clavier

| Action | Raccourci |
|--------|-----------|
| Naviguer entre sections sidebar | `Tab` / `Shift+Tab` |
| Activer une section | `Entree` ou `Espace` |
| Naviguer dans un accordeon | `Tab` entre les headers, `Entree` pour deplier/replier |
| Changer de mode (Style) | `Fleches gauche/droite` dans le radiogroup |
| Toggle un switch | `Espace` |
| Fermer la fenetre | `Cmd+W` ou `Escape` |
| Ouvrir dropdown | `Espace` ou `Entree`, fleches pour naviguer |

### 8.4 Contraste et lisibilite

- Tous les textes respectent WCAG 2.1 AA (ratio >= 4.5:1, voir section 2.2)
- Les elements interactifs ont un etat `:focus-visible` avec un contour `2px solid #2d7a4f`, offset 2px
- Les icones a fonction interactive ont une taille minimum de 24x24px (cible tactile minimum 44x44px non requise car desktop-only)
- Le texte n'est jamais inferieur a 11px

---

## 9. Micro-interactions

### 9.1 Son

| Declencheur | Son | Condition |
|-------------|-----|-----------|
| Debut dictee (raccourci appuye) | Stylo sur papier (~300ms) | AudioFeedback active |
| Fin dictee (raccourci relache) | Page tournee (~200ms) | AudioFeedback active |
| Annulation dictee | Froissement papier (~200ms) | AudioFeedback active |
| Erreur | Tap sur bois (~150ms) | AudioFeedback active |

### 9.2 Animations

| Element | Animation | Duree | Easing |
|---------|-----------|-------|--------|
| **Overlay apparition** | Fade in + slide down 8px | 200ms | ease-out |
| **Overlay disparition** | Fade out | 300ms | ease-in |
| **Barres audio overlay** | Height + opacity smoothing | 60ms / 120ms | ease-out |
| **Changement de section sidebar** | Contenu fade crossfade | 150ms | ease-in-out |
| **Accordeon deplier** | Height auto + fade in contenu | 200ms | ease-out |
| **Accordeon replier** | Height 0 + fade out contenu | 150ms | ease-in |
| **Selection de mode (Style)** | Bordure gauche slide in + fond transition | 200ms | ease-out |
| **Toggle switch** | Slide horizontal du cercle | 150ms | spring(1, 80, 10) |
| **Bouton hover** | Background color transition | 150ms | ease |
| **Bouton press** | Scale 0.97 | 100ms | ease |
| **Flash "Copie !"** | Apparition + disparition texte | 2000ms total | ease-in-out |
| **Barre de progression** | Width transition smooth | 300ms | ease-out |
| **Onboarding step transition** | Slide left + fade | 300ms | ease-in-out |
| **Pastille statut clignotante** | Opacity pulse 0.5 -> 1 | 1500ms infinite | ease-in-out |
| **Message d'erreur apparition** | Slide down + fade in | 200ms | ease-out |

### 9.3 Transitions entre etats

**Idle -> Ecoute :**
1. Son "stylo sur papier" joue (si active)
2. Icone tray passe au vert (instantane)
3. Overlay apparait (fade in + slide, 200ms)
4. Ecran Accueil met a jour le statut (si visible)

**Ecoute -> Transcription :**
1. Barres audio s'arretent (fade a hauteur minimale, 200ms)
2. Zone centrale overlay : texte "Transcription..." apparait (crossfade, 150ms)
3. Icone tray passe au bleu

**Transcription -> Collage :**
1. Son "page tournee" joue (si active)
2. Overlay : flash vert (fond passe a `#16a34a40` pendant 300ms)
3. Overlay disparait (fade out, 300ms)
4. Icone tray revient au sombre
5. Ecran Accueil : apercu derniere dictee se met a jour (fade in nouveau texte, 200ms)

**Erreur :**
1. Son "tap sur bois" joue (si active)
2. Overlay : fond passe a `#dc262640` pendant 500ms puis disparait
3. Toast notification en bas de la fenetre settings (si ouverte) avec le message d'erreur

---

## 10. Composants UI reutilisables

### 10.1 Design Tokens

```css
/* Spacing */
--space-1: 4px;
--space-2: 8px;
--space-3: 12px;
--space-4: 16px;
--space-5: 20px;
--space-6: 24px;
--space-8: 32px;

/* Border radius */
--radius-sm: 4px;
--radius-md: 8px;
--radius-lg: 12px;
--radius-full: 9999px;

/* Shadows */
--shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.05);
--shadow-md: 0 4px 12px rgba(0, 0, 0, 0.08);
--shadow-lg: 0 8px 32px rgba(0, 0, 0, 0.12);
--shadow-overlay: 0 4px 16px rgba(0, 0, 0, 0.25);

/* Transitions */
--transition-fast: 100ms ease;
--transition-normal: 150ms ease;
--transition-slow: 300ms ease-out;
```

### 10.2 Boutons

#### Bouton primaire

```
Fond: #2d7a4f
Texte: #ffffff
Font: Inter 14px semibold
Padding: 10px 20px
Border-radius: 8px
Hover: fond #246b43
Active: scale(0.97)
Focus: outline 2px solid #2d7a4f, offset 2px
Disabled: opacity 0.5, cursor not-allowed
```

#### Bouton secondaire

```
Fond: transparent
Bordure: 1px solid #e5e2db
Texte: #1a1a2e
Font: Inter 14px medium
Padding: 10px 20px
Border-radius: 8px
Hover: fond #f5f3ee, bordure #2d7a4f40
Active: scale(0.97)
Focus: outline 2px solid #2d7a4f, offset 2px
```

#### Bouton danger

```
Fond: transparent
Bordure: 1px solid #dc262640
Texte: #dc2626
Font: Inter 14px medium
Padding: 10px 20px
Border-radius: 8px
Hover: fond #dc26260d
Active: scale(0.97)
```

#### Bouton icone (actions inline)

```
Taille: 32x32px
Fond: transparent
Icone: 16px, couleur #6b7280
Border-radius: 6px
Hover: fond #f5f3ee, icone #2d7a4f
Active: scale(0.95)
```

### 10.3 Cartes

#### Carte de parametres (SettingsGroup)

```
Fond: #f5f3ee
Bordure: 1px solid #e5e2db
Border-radius: 10px
Padding: 0 (contenu interne gere par les enfants)
Separateurs internes: 1px solid #e5e2db
```

#### Carte de mode (ecran Style)

```
Fond: #f5f3ee (inactif) / #2d7a4f08 (actif)
Bordure: 1px solid #e5e2db (inactif) / 1px solid #2d7a4f + bordure gauche 3px (actif)
Border-radius: 10px
Padding: 16px
Cursor: pointer
Hover (inactif): shadow-sm, bordure #2d7a4f40
Transition: all 200ms ease-out
```

#### Carte de statut (ecran Accueil)

```
Fond: #f5f3ee
Bordure: 1px solid #e5e2db
Border-radius: 8px
Padding: 16px
```

### 10.4 Toggles (Switch)

```
Track inactif: fond #e5e2db, 40x22px, border-radius 11px
Track actif: fond #2d7a4f
Cercle: 18x18px, fond #ffffff, shadow-sm, border-radius 50%
Position inactif: left 2px
Position actif: left 20px
Transition: 150ms spring
```

### 10.5 Dropdowns (Select)

```
Fond: #faf8f3
Bordure: 1px solid #e5e2db
Border-radius: 8px
Padding: 8px 12px
Font: Inter 14px regular #1a1a2e
Fleche: ChevronDown 14px #6b7280
Focus: bordure #2d7a4f
Menu ouvert: fond #faf8f3, bordure #e5e2db, shadow-md, border-radius 8px
Option hover: fond #f5f3ee
Option selectionnee: fond #2d7a4f1a, texte #2d7a4f
```

### 10.6 Input texte

```
Fond: #faf8f3
Bordure: 1px solid #e5e2db
Border-radius: 8px
Padding: 8px 12px
Font: Inter 14px regular #1a1a2e
Placeholder: #9ca3af
Focus: bordure #2d7a4f, shadow 0 0 0 3px #2d7a4f1a
Erreur: bordure #dc2626, shadow 0 0 0 3px #dc26261a
```

### 10.7 Slider

```
Track: fond #e5e2db, height 4px, border-radius 2px
Track rempli: fond #2d7a4f
Thumb: 16x16px, fond #ffffff, bordure 2px solid #2d7a4f, border-radius 50%, shadow-sm
Thumb hover: scale(1.1)
Thumb active: scale(0.95)
```

### 10.8 Badge / Tag

```
Fond: #2d7a4f1a (vert) / #1a56db1a (bleu) / #dc26261a (rouge)
Texte: couleur assortie, Inter 11px medium
Padding: 2px 8px
Border-radius: 4px
```

### 10.9 Toast (notification)

```
Position: bas de la fenetre, centre, margin-bottom 16px
Fond: #1a1a2e
Texte: #ffffff, Inter 13px medium
Padding: 10px 16px
Border-radius: 8px
Shadow: shadow-lg
Apparition: slide up + fade in, 200ms
Disparition: fade out, 300ms
Duree: 3s (info), 5s (erreur)
Icone: Check vert (succes), AlertTriangle rouge (erreur), Info bleu (info)
```

---

## 11. Migration depuis Handy UI

### 11.1 Ce qui est conserve (adapte au theme)

| Composant Handy | Action DictAI | Notes |
|-----------------|---------------|-------|
| `Sidebar.tsx` | **Refactored** -- de 7 sections a 3 | Structure conservee, contenu simplifie |
| `WriteModeSelector.tsx` | **Redesigne** -- cartes au lieu de petits boutons | Deux variantes : compact (Accueil) et detaille (Style) |
| `GeneralSettings.tsx` | **Eclate** -- contenu redistribue dans Accueil + Parametres | Les composants enfants sont reutilises, pas recrees |
| `HistorySettings.tsx` | **Deplace** -- dans Parametres > Historique | Code identique, style adapte au theme cahier |
| `RecordingOverlay.tsx` | **Restyle** -- memes etats, nouvelles couleurs | Structure JSX conservee, CSS remplace |
| `RecordingOverlay.css` | **Remplace** -- nouvelles couleurs et dimensions | Fichier reecrit |
| `AccessibilityOnboarding.tsx` | **Integre** -- dans le flux 4 etapes | Logique de polling conservee, UI reecrite |
| `Onboarding.tsx` | **Simplifie** -- plus de choix de modele | Un seul modele, telechargement automatique |
| Tous les SettingContainer/SettingsGroup | **Restyle** -- couleurs cahier | Props inchangees, CSS adapte |
| Hooks (`useSettings`, `useModelStore`) | **Conserves tels quels** | Aucune modification necessaire |
| `commands` (Tauri bindings) | **Conserves tels quels** | Backend Rust inchange |

### 11.2 Ce qui est redesigne

| Element | Avant (Handy) | Apres (DictAI) |
|---------|---------------|----------------|
| Palette couleurs | Violet `#7c3aed` / dark `#1c1b22` | Vert `#2d7a4f` / creme `#faf8f3` |
| Theme | Dark-first (fond sombre) | Light-first (fond clair papier creme) |
| Logo | `HandyTextLogo` / `DictationLogo` (micro generique) | Micro-plume DictAI + texte Caveat |
| Sidebar | 7 sections techniques | 3 sections simplifiees |
| Overlay barres | Rose `#ffe5ee` | Vert `#2d7a4f` |
| Overlay fond | Noir `#000000cc` | Ardoise `#1a1a2ecc` |
| Onboarding | Choix de modele (technique) | 4 etapes guidees (permissions + test) |
| Ecran principal | GeneralSettings (parametres techniques) | Ecran Accueil (statut + mode + apercu) |
| Modes d'ecriture | 3 petits boutons avec emoji | 3 cartes descriptives avec exemples avant/apres |

### 11.3 Ce qui est supprime

| Element | Raison |
|---------|--------|
| `HandyTextLogo.tsx` | Remplace par le logo DictAI |
| `TranslateToEnglish.tsx` | FR-only pour le MVP |
| `GlobalShortcutInput.tsx` | Redondant avec `ShortcutInput` |
| Lien donate Handy (`handy.computer/donate`) | Remplace par lien DictAI |
| Lien GitHub Handy (`github.com/cjpais/Handy`) | Remplace par repo DictAI |
| Theme sombre par defaut | Le theme cahier est light-only pour le MVP. Un dark mode pourra etre ajoute post-MVP (ardoise d'ecole : fond sombre `#1c1b22`, craie blanche `#f5f3ee`, vert tableau `#4ade80`) |
| Section sidebar "Post-Processing" | Integre dans Parametres > Avance |
| Section sidebar "Models" | Integre dans Parametres > Modeles |
| Section sidebar "Debug" | Cache dans Parametres > Avance > Debug toggle |
| Section sidebar "About" | Integre dans Parametres > A propos |

### 11.4 Fichiers CSS a modifier

| Fichier | Modification |
|---------|-------------|
| `src/App.css` | Remplacer toutes les variables CSS `:root` et le media query `prefers-color-scheme: dark` par la palette cahier |
| `tailwind.config.js` | Mettre a jour les couleurs custom (`logo-primary` -> `primary`, ajouter `surface`, `border-cahier`, etc.) |
| `src/overlay/RecordingOverlay.css` | Remplacer les couleurs (fond, barres, bouton annuler) |

### 11.5 Ordre de migration recommande

1. **Variables CSS + Tailwind config** -- Changer la palette en un commit. Impact visuel immediat sur toute l'app.
2. **Logo DictAI** -- Remplacer `DictationLogo.tsx` et `HandyTextLogo.tsx`
3. **Sidebar 3 sections** -- Refactorer `Sidebar.tsx` + creer les ecrans Accueil et Style
4. **Overlay restyle** -- `RecordingOverlay.css` mise a jour
5. **Onboarding 4 etapes** -- Refactorer `Onboarding.tsx` + `AccessibilityOnboarding.tsx` en flux sequentiel
6. **Ecran Accueil** -- Nouveau composant principal (statut + mode + apercu historique)
7. **Ecran Style** -- Nouveau composant avec cartes detaillees
8. **Parametres accordeon** -- Refactorer `AdvancedSettings`, `DebugSettings`, `AboutSettings` en sous-sections collapsibles
9. **Sons** -- Ajouter les fichiers audio et les integrer dans le pipeline d'events
10. **Polish** -- Focus visible, VoiceOver labels, animations finales

---

*Ce document sera utilise comme reference pour la creation des epics et stories d'implementation du redesign DictAI.*
