# Dictation IA Locale

Application de dictée / transcription vocale local-first inspirée par Wispr Flow. Transcription fidèle + post-traitement IA pour un texte propre et personnalisé.

## ✨ Concept

**"Speak naturally, write perfectly"**

Une dictée qui va plus loin que la simple transcription : un pipeline local qui capture ta voix, la transforme en texte, puis l'adapte selon ton contexte (pro, perso, code) avant de l'injecter dans n'importe quelle application.

## 🎯 Fonctionnalités

### MVP (Phase 1)
- [ ] Raccourci clavier global pour démarrer/arrêter l'enregistrement
- [ ] Transcription locale avec Whisper
- [ ] Post-traitement IA (nettoyage, ponctuation)
- [ ] Collage automatique au curseur
- [ ] Historique minimal

### "Magie" (Phase 2)
- **Nettoyage intelligent** : suppression des "euh", répétitions, phrases cassées
- **Structuration** : paragraphes, listes, titres Markdown
- **Modes d'écriture** :
  - 🗣️ **Chat** : ton naturel, peu de reformulation
  - 💼 **Pro** : concis, poli, style email
  - 💻 **Code** : conserve le jargon technique, formate en Markdown
- **Dictionnaire personnel** : noms propres, jargon, acronymes
- **Profils** : contextes adaptés (travail / perso / enseignement / dev)

## 🏗️ Architecture

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐    ┌─────────────┐
│   Touche    │───▶│    Audio     │───▶│     STT     │───▶│     LLM     │
│  raccourci  │    │   (micro)    │    │  (Whisper)  │    │ (nettoyage) │
└─────────────┘    └──────────────┘    └─────────────┘    └──────┬──────┘
                                                                  │
                                                                  ▼
                                                           ┌─────────────┐
                                                           │   Collage   │
                                                           │  (cursor)   │
                                                           └─────────────┘
```

### Stack technique (proposition)
- **Backend** : Python (itération rapide) ou Swift (intégration macOS native)
- **STT** : Whisper (ou variante optimisée locale)
- **LLM** : Ollama (local) ou option cloud
- **Stockage** : SQLite (profils, dictionnaire, préférences)

## 📋 Roadmap

| Phase | Objectif | Durée estimée |
|-------|----------|---------------|
| 0 | Cadrage MVP + modes + politique données | 1-2 jours |
| 1 | MVP local : enregistrement → transcription → collage | 1-2 semaines |
| 2 | Nettoyage LLM + profils + dictionnaire | 1-2 semaines |
| 3 | Qualité produit : VAD, UI menu bar, tests multi-apps | 2-4 semaines |
| 4 | Finitions : streaming, commandes vocales, export | Continu |

## 🔒 Privacy-first

- Fonctionnement **local par défaut** : audio et textes restent sur ta machine
- Stockage minimal : option pour ne rien conserver ou historique chiffré
- Mode cloud **opt-in explicite** avec transparence totale
- Bouton "tout effacer" accessible

## 🚀 Démarrage rapide

```bash
# Cloner le repo
git clone git@github.com:Uhama91/dictation-ia-locale.git
cd dictation-ia-locale

# Installation (à compléter selon la stack choisie)
# TODO

# Lancer l'application
# TODO
```

## 📝 License

MIT

---

*Projet en cours de développement — local-first, privacy-focused.*
