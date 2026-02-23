# Spécification fonctionnelle

## User Stories

### MVP - Phase 1

**US-001** : En tant qu'utilisateur, je veux démarrer l'enregistrement via un raccourci clavier global pour ne pas avoir à quitter mon application en cours.

**Critères d'acceptation:**
- Raccourci configurable (défaut : double-tap sur une touche ou combo)
- Feedback visuel/sonore immédiat
- Fonctionne dans n'importe quelle application

---

**US-002** : En tant qu'utilisateur, je veux que l'audio soit transcrit localement pour garantir la confidentialité de mes données.

**Critères d'acceptation:**
- Transcription sans envoi de données externes (mode local)
- Support du français et de l'anglais
- Temps de transcription < 2x la durée audio

---

**US-003** : En tant qu'utilisateur, je veux que le texte final soit collé automatiquement à l'endroit où se trouve mon curseur.

**Critères d'acceptation:**
- Détection de l'application active
- Simulation de frappe ou utilisation du presse-papiers
- Fonctionne dans les principales apps (Chrome, VSCode, Notion, Mail)

---

### Phase 2 - Nettoyage IA

**US-004** : En tant qu'utilisateur, je veux choisir un "mode" d'écriture pour adapter le style du texte généré.

**Modes:**
- **Chat** : Ton conversationnel, minimal de reformulation
- **Pro** : Style email professionnel, concis
- **Code** : Préservation du jargon technique, formatage Markdown

---

**US-005** : En tant qu'utilisateur, je veux pouvoir définir un dictionnaire personnel pour que l'IA reconnaisse mes termes spécifiques.

**Fonctionnalités:**
- Ajout de mots/phrases
- Catégorisation (noms propres, jargon, acronymes)
- Import/export

---

## Règles métier

### Mode Chat
- Corriger l'orthographe uniquement si évidente
- Ajouter ponctuation basique
- Conserver le ton et la structure

### Mode Pro
- Reformuler pour clarté et concision
- Ton poli mais pas trop formel
- Structure en paragraphes clairs

### Mode Code
- Ne jamais traduire les termes techniques anglais
- Préserver les symboles et identifiants
- Formater les structures ("si X alors Y" → pseudo-code)
