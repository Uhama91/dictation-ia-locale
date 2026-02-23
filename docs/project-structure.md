# Structure du projet

```
dictation-ia-locale/
├── README.md                 # Vue d'ensemble du projet
├── TODO.md                   # Tâches et roadmap
├── LICENSE                   # Licence MIT
├── package.json              # Métadonnées du projet
│
├── docs/                     # Documentation
│   ├── adr/                  # Architecture Decision Records
│   │   └── 001-backend-language.md
│   ├── specs/                # Spécifications
│   │   └── functional-spec.md
│   └── dev-log.md            # Journal de développement
│
├── src/                      # Code source (à créer)
│   ├── audio/                # Capture audio
│   ├── stt/                  # Speech-to-text
│   ├── llm/                  # Post-traitement LLM
│   ├── input/                # Raccourcis clavier
│   └── ui/                   # Interface utilisateur
│
├── tests/                    # Tests
│
├── config/                   # Fichiers de configuration
│   └── default-profiles.json # Profils par défaut
│
└── assets/                   # Ressources (sons, icônes)
```
