# Quick Spec + Dev Story 8.1 — Structuration intelligente du texte transcrit

**Date** : 2026-03-02
**Statut** : Validé — review Gemini intégrée (4 corrections)
**Auteurs** : Party mode BMAD (John PM, Winston Architect, Barry Dev, Mary Analyst)

---

## QUICK SPEC

### Contexte & problème

DictAI transcrit fidèlement la parole mais produit du texte brut non structuré. Wispr Flow, concurrent direct, structure automatiquement le texte en paragraphes, listes et messages courts selon l'intention de l'utilisateur — sans action supplémentaire.

Actuellement, DictAI :
- Post-traite via des règles locales (filler words, ponctuation) + LLM optionnel (Ollama qwen2.5:0.5b)
- N'a aucune logique de structuration (paragraphes, listes)
- Plafonne à 128 tokens de sortie (trop peu pour du texte long structuré)
- Ne détecte pas l'intention de l'utilisateur (liste vs paragraphe vs message court)

### Objectif

Ajouter une couche de **structuration automatique universelle** qui :
- S'applique dans les **3 modes** (Chat, Pro, Code)
- Détecte l'intention structurelle **avant le LLM** (règles FR = 0 latence)
- Adapte le **format de sortie** selon l'intention détectée
- Différencie les modes uniquement sur le **ton** (informel / professionnel / technique), pas sur la structure

### Comportements cibles par type de sortie

| Type | Déclencheur | Exemple input oral | Output attendu |
|------|-------------|-------------------|----------------|
| **Message court** | < 20 mots, aucun marqueur | *"Rappelle-moi d'appeler Pierre demain"* | Texte inline, ponctué |
| **Paragraphe unique** | 20–60 mots, 1 sujet | *"Je voulais te dire que le rapport est prêt, j'ai vérifié tous les chiffres"* | Un bloc de texte propre |
| **Multi-paragraphes** | > 60 mots + marqueur de pivot | *"D'un côté... par contre de l'autre..."* | Texte avec `\n\n` entre sujets |
| **Liste** | Marqueurs énumératifs FR détectés | *"D'abord le lait, ensuite du pain, enfin des œufs"* | `- Lait\n- Pain\n- Œufs` |

### Différence de ton par mode (structure identique)

| Structure | Chat (informel) | Pro (professionnel) | Code (technique) |
|-----------|-----------------|---------------------|------------------|
| Liste | `- j'ai besoin de lait` | `- Lait` | `- lait` (verbatim) |
| Paragraphe | Conserve la formulation orale, nettoie | Reformule en style email | Conserve jargon technique |
| Message court | Minimal, conservé | Concis, formel | Verbatim corrigé |

### Marqueurs FR à détecter (Tier 1 = fort, Tier 2 = moyen, Tier 3 = faible)

**Listes — Tier 1 (2 marqueurs = liste confirmée) :**
`premièrement`, `deuxièmement`, `troisièmement`, `d'une part`, `d'autre part`, `en premier lieu`, `en deuxième lieu`, `en troisième lieu`

**Listes — Tier 2 (2 marqueurs = liste confirmée) :**
`d'abord`, `ensuite`, `puis`, `enfin`, `en dernier lieu`, `finalement`, `pour commencer`, `pour finir`, `dans un premier temps`, `dans un deuxième temps`

**Listes — Tier 3 (3+ marqueurs = liste probable) :**
`également`, `de plus`, `en outre`, `par ailleurs`, `et aussi`, `sans oublier`, `et puis`

**Pivot de paragraphe (double condition obligatoire) :**
`par contre`, `cependant`, `néanmoins`, `toutefois`, `sur un autre sujet`, `pour ce qui est de`, `passons à`, `autre chose`, `autre point important`, `à propos de`, `s'agissant de`

⚠️ Mots exclus car trop courants dans la parole normale : `maintenant`, `concernant` (Gemini feedback)

**Condition de déclenchement pivot :** marqueur de pivot détecté **ET** (`word_count > 60` OU ponctuation forte avant le marqueur — `.` ou `,` généré par Whisper). Sans cette double condition, trop de faux positifs sur des phrases conversationnelles normales.

**Jamais restructuré en liste (règle anti-question affinée) :**
- Messages < 15 mots qui sont une pure question sans marqueurs de liste (ex: *"Comment tu vas ?"*)
- La règle anti-question s'applique **uniquement aux `SingleMessage` purs** sans marqueurs Tier 1/2
- ⚠️ Exception : si une question est suivie de marqueurs de liste forts (ex: *"Quelles sont les étapes ? D'abord X, ensuite Y, enfin Z"*), la structuration en liste est appliquée normalement

### Architecture technique (3 couches)

```
Layer 0 — rules::apply() [existant, inchangé]
          → filler words, élisions, ponctuation, bégaiements

Layer 1 — rules::detect_structure() [NOUVEAU, règles pures, < 1ms]
          → StructureHint { SingleMessage | Paragraph | MultiParagraph | List }
          → Entrée : texte post-règles + word_count

Layer 2 — llm::cleanup::run() [MODIFIÉ]
          → system_prompt() reçoit StructureHint → ton + format dans le prompt
          → num_predict adaptatif (non plus 128 fixe)
          → always_use_llm() → true pour TOUS les modes si structure détectée

Layer 3 — rules::apply_structure_fallback() [NOUVEAU]
          → Si LLM indisponible ET List détecté : formatter regex MINIMAL
          → Insère \n- DEVANT chaque marqueur Tier 1/2 (ne retire PAS les mots de liaison)
          → "d'abord X ensuite Y enfin Z" → "- d'abord X\n- ensuite Y\n- enfin Z"
          → Objectif : structure lisible garantie, réécriture confiée au LLM uniquement
          → Garantit les listes même sans Ollama
```

### Routage modifié

**Règle actuelle :** fast-path si `confidence >= 0.82 AND mots <= 30 AND mode != Pro`

**Nouvelle règle :** fast-path si `confidence >= 0.82 AND mots <= 30 AND mode != Pro AND structure == SingleMessage`
- Tout texte avec `StructureHint::List | MultiParagraph` force le LLM (ou fallback Layer 3)
- `Paragraph` suit la règle existante (court + confiance élevée = fast-path OK)

### Tokens adaptatifs

```rust
fn compute_num_predict(word_count: usize, hint: StructureHint) -> i64 {
    match hint {
        StructureHint::SingleMessage => 64,
        StructureHint::Paragraph     => 128,
        StructureHint::MultiParagraph => (word_count as i64 * 2 + 40).min(384),
        StructureHint::List          => (word_count as i64 * 2 + 30).min(256),
    }
}
```

### Fichiers modifiés (4)

| Fichier | Type | Changement |
|---------|------|------------|
| `src-tauri/src/pipeline/rules.rs` | Modifié | + `detect_structure()` + `apply_structure_fallback()` + `StructureHint` enum |
| `src-tauri/src/pipeline/modes.rs` | Modifié | `system_prompt(hint)` avec `StructureHint`, tous modes `always_use_llm → true` si hint ≠ SingleMessage |
| `src-tauri/src/pipeline/orchestrator.rs` | Modifié | Passe `StructureHint` au routage et au LLM, override fast-path si structure ≠ SingleMessage |
| `src-tauri/src/llm/cleanup.rs` | Modifié | `num_predict` adaptatif, `build_ollama_payload` prend `StructureHint` |

### Non-functional requirements implicites

- **Latence** : `detect_structure()` < 1ms (pur regex, pas de LLM)
- **Fallback sans Ollama** : listes produites par Layer 3 même si Ollama KO
- **Régression nulle** : les 142 tests Rust existants doivent passer
- **Prompt budget** : system prompts ≤ 60 tokens (contrainte qwen2.5:0.5b)
- **Backward compat** : `PipelineResult` struct inchangée côté frontend

---

## DEV STORY 8.1

**En tant qu'utilisateur de DictAI,**
**je veux** que mon texte dicté soit automatiquement structuré en paragraphes, listes ou message court selon mon intention,
**afin** de coller directement dans n'importe quelle application sans reformatage manuel.

### Acceptance Criteria

**AC1 — Détection de liste (marqueurs FR)**
- Given une transcription contenant ≥ 2 marqueurs Tier 1 ou Tier 2 (ex: *"d'abord... ensuite... enfin"*)
- When le pipeline de post-traitement s'exécute
- Then le texte est formaté en liste à tirets (`- item\n- item\n- item`)
- And ce comportement s'applique dans les 3 modes (Chat, Pro, Code)

**AC2 — Détection multi-paragraphes**
- Given une transcription > 60 mots contenant ≥ 1 marqueur de pivot (ex: *"par contre"*, *"concernant"*)
- When le pipeline s'exécute
- Then le texte est découpé en ≥ 2 paragraphes séparés par `\n\n`

**AC3 — Message court préservé**
- Given une transcription < 20 mots sans marqueur de liste ni de pivot
- When le pipeline s'exécute
- Then le texte sort en une seule ligne, sans structure forcée
- And le fast-path (règles seules) reste actif si confiance >= 0.82

**AC4 — Fallback sans Ollama**
- Given Ollama est indisponible
- And la transcription contient des marqueurs de liste Tier 1 ou Tier 2
- When le pipeline s'exécute
- Then le Layer 3 (formatter regex) produit une liste à tirets correcte
- And `PipelineResult.llm_fallback = true`

**AC5 — Différence de ton par mode (structure identique)**
- Given le même input énumératif dicté
- When traité en mode Chat vs Pro
- Then la structure (liste) est identique dans les deux cas
- And le ton diffère : Chat conserve la formulation orale, Pro capitalise et formate formellement

**AC6 — Questions simples non restructurées (règle affinée)**
- Given une transcription qui est une **pure question courte** (< 15 mots, se termine par `?`, sans marqueurs Tier 1/2)
- When le pipeline s'exécute
- Then aucune restructuration en liste n'est appliquée — message conservé inline
- But Given une question suivie de marqueurs de liste forts ("Quelles sont les étapes ? D'abord X, ensuite Y, enfin Z")
- Then la liste est structurée normalement (la question ne bloque pas les marqueurs Tier 1/2)

**AC7 — Tokens adaptatifs**
- Given une transcription longue (> 60 mots) avec `StructureHint::MultiParagraph`
- When le payload Ollama est construit
- Then `num_predict` est > 128 (adaptatif selon longueur)

**AC8 — Régression tests**
- Given les 142 tests Rust existants (`cargo test --lib`)
- When la feature est implémentée
- Then tous les 142 tests passent (zéro régression)

### Tasks / Subtasks

- [ ] **Task 1 — `StructureHint` + `detect_structure()` dans `rules.rs`** (AC: 1, 2, 3, 6)
  - [ ] 1.1 Définir enum `StructureHint { SingleMessage, Paragraph, MultiParagraph, List }`
  - [ ] 1.2 Implémenter `detect_structure(text: &str) -> StructureHint` avec regex Tier 1/2/3
  - [ ] 1.3 Implémenter `is_pure_question()` : pure question courte (< 15 mots, `?` final, sans marqueurs Tier 1/2)
  - [ ] 1.4 Écrire tests unitaires : liste Tier1, liste Tier2, pivot paragraphe, question pure, question+liste, message court
  - [ ] 1.5 Implémenter `apply_structure_fallback(text, hint) -> String` : insère `\n- ` devant marqueurs (sans retirer les mots)
  - [ ] 1.6 Écrire tests fallback : "d'abord X ensuite Y enfin Z" → "- d'abord X\n- ensuite Y\n- enfin Z"
  - [ ] 1.7 Détection pivot paragraphe avec double condition : marqueur ET (word_count > 60 OU ponctuation forte avant)

- [ ] **Task 2 — Prompts restructurés dans `modes.rs`** (AC: 5)
  - [ ] 2.1 Modifier signature `system_prompt(&self, hint: StructureHint) -> &'static str` (ou `String`)
  - [ ] 2.2 Réécrire prompt **Chat** : ton informel + micro-exemple few-shot liste, ≤ 60 tokens
  - [ ] 2.3 Réécrire prompt **Pro** : ton formel + micro-exemple few-shot liste capitalisée, ≤ 60 tokens
  - [ ] 2.4 Réécrire prompt **Code** : jargon préservé + structure basique, ≤ 60 tokens
  - [ ] 2.5b Tester empiriquement les 3 prompts avec qwen2.5:0.5b (valider format `- ` vs `1.` en sortie)
  - [ ] 2.5 Mettre à jour `always_use_llm()` : retourne `true` si hint ≠ `SingleMessage`
  - [ ] 2.6 Mettre à jour les tests `modes.rs` existants (signature changée)

- [ ] **Task 3 — Routage dans `orchestrator.rs`** (AC: 3, 7)
  - [ ] 3.1 Appeler `detect_structure()` après `rules::apply()`
  - [ ] 3.2 Modifier `route()` : override fast-path si `hint == List || hint == MultiParagraph`
  - [ ] 3.3 Passer `hint` à `llm_cleanup_fn` (mettre à jour signature du callback)
  - [ ] 3.4 Appeler `apply_structure_fallback()` en cas de `llm_fallback && hint == List`
  - [ ] 3.5 Mettre à jour les tests `orchestrator.rs` existants

- [ ] **Task 4 — Tokens adaptatifs dans `cleanup.rs`** (AC: 7)
  - [ ] 4.1 Implémenter `compute_num_predict(word_count, hint) -> i64`
  - [ ] 4.2 Modifier `build_ollama_payload()` pour prendre `hint` et appeler `compute_num_predict`
  - [ ] 4.3 Modifier `run()` pour prendre `hint: StructureHint` en paramètre
  - [ ] 4.4 Mettre à jour les tests `cleanup.rs` existants
  - [ ] 4.5 Vérifier que `cargo test --lib` → 142 tests OK (AC8)

### Exemples de prompts cibles (≤ 60 tokens, avec micro-exemple few-shot)

> ⚠️ Recommandation Gemini : qwen2.5:0.5b réagit mieux aux **exemples concrets** qu'aux instructions abstraites. Chaque prompt inclut un micro-exemple pour forcer le format `- ` (et non `1.` ou numérotation).

**Chat + List hint :**
```
Tu corriges les transcriptions vocales françaises.
Exemple: "d'abord faire la vaisselle ensuite nettoyer le sol enfin ranger"
→ "- D'abord faire la vaisselle\n- Ensuite nettoyer le sol\n- Enfin ranger"
Conserve le ton oral. Réponds uniquement avec le texte corrigé.
```

**Pro + List hint :**
```
Tu es un rédacteur professionnel.
Exemple: "d'abord faire la vaisselle ensuite nettoyer le sol enfin ranger"
→ "- Faire la vaisselle\n- Nettoyer le sol\n- Ranger"
Reformule en français formel. Réponds uniquement avec le texte reformulé.
```

**Code + hint quelconque :**
```
Tu es un assistant technique.
Corrige la ponctuation. Préserve TOUS les termes techniques anglais et symboles.
Ne traduis jamais le jargon. Structure si énumération explicite.
Réponds uniquement avec le texte corrigé.
```

> Note : Les prompts exacts devront être **testés empiriquement** avec qwen2.5:0.5b avant validation finale (Task 2.2 à 2.4). Le few-shot peut nécessiter ajustement selon les résultats réels.

### Risques identifiés

| Risque | Probabilité | Mitigation |
|--------|-------------|------------|
| qwen2.5:0.5b ignore les instructions de structuration dans le prompt | Moyenne | Layer 3 fallback garantit les listes même si LLM échoue |
| Prompt > 60 tokens → modèle ignore fin du prompt | Haute | Contraindre prompts, tester empiriquement |
| Faux positifs listes (marqueurs dans un contexte non-liste) | Faible | Seuils Tier 1/2/3 calibrés, question detection exclut les FP principaux |
| Régression sur tests existants (signature changée) | Certaine | Updater tous les call sites + tests en Task 2/3/4 |

### Definition of Done

- [ ] `cargo test --lib` → 142+ tests, 0 failed
- [ ] Tests manuels : liste (d'abord/ensuite/enfin) → tirets ✓
- [ ] Tests manuels : texte long + "par contre" → 2 paragraphes ✓
- [ ] Tests manuels : question pure → pas de restructuration ✓
- [ ] Tests manuels : question + liste → liste structurée ✓
- [ ] Tests manuels : sans Ollama → fallback liste lisible ✓
- [ ] Prompts Chat/Pro/Code validés empiriquement avec qwen2.5:0.5b (format `- ` confirmé)

---

## REVIEW GEMINI — Résumé des corrections intégrées

**Source** : Review externe Gemini 2.5 Pro, 2026-03-02
**Verdict global** : Architecture validée ✅ — 4 points d'attention corrigés

| # | Point Gemini | Correction apportée |
|---|-------------|---------------------|
| 1 | Layer 3 trop ambitieux — extraire le contenu entre marqueurs par regex est fragile | Fallback minimal : insère `\n- ` devant les marqueurs, conserve les mots de liaison. Réécriture = LLM uniquement. |
| 2 | qwen2.5:0.5b réagit mieux aux exemples few-shot qu'aux instructions abstraites | Micro-exemple ajouté dans chaque prompt Chat/Pro + tâche de test empirique ajoutée |
| 3 | Règle anti-question trop large — bloque des listes légitimes | Anti-question cantonnée aux `SingleMessage` purs. Exception : marqueurs Tier 1/2 présents → liste appliquée |
| 4 | Mots pivots trop courants dans la parole normale (`maintenant`, `concernant`) | Double condition obligatoire : marqueur pivot + (word_count > 60 OU ponctuation forte). Exclusion de `maintenant`/`concernant`. |
