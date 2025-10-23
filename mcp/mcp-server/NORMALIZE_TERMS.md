# Tool: normalize_terms (LLM→JSON)

## Vue d'ensemble

Le tool `normalize_terms` convertit un transcript en langage naturel en un DomainModel JSON strict via LLM.

## Architecture

```
Transcript (texte) → LLM Router → DomainModel JSON → Validation Schema → Result
```

## Implémentation

### Handler MCP (`handle_tool_call`)

```rust
"normalize_terms" => {
    let input_lang = params.get("input_lang")...
    let transcript = params.get("transcript")...
    normalize_terms_with_llm(input_lang, transcript).await?
}
```

### Fonction principale (`normalize_terms_with_llm`)

**Étapes :**

1. **Chargement .env** : Load environment variables for LLM provider
2. **System Prompt** : Génère un prompt strict selon la langue (fr/en)
3. **Appel LLM** : 
   - Si `LLM_PROVIDER=ollama`: POST `/api/generate`
   - Sinon: Appel API externe (OpenAI, etc.)
4. **Parse JSON** : Convertit la réponse LLM en JSON
5. **Validation** : Valide contre le JSON Schema via `validate_domain_model()`
6. **Résultat** : Retourne le DomainModel validé ou erreur

### System Prompt

**Français :**
```
Tu es un normalizer de Domain Model. Rends UNIQUEMENT un JSON valide 
DomainModel conforme au schema. Interdis les champs non listés.

Schema DomainModel (STRICT):
{
  "entities": [...],
  "relations": [...],
  "invariants": [...]
}

RÈGLES STRICTES:
1. AUCUN champ en dehors de ce schema
2. Tous les champs obligatoires DOIVENT être présents
3. Les types enum DOIVENT correspondre exactement
4. Réponds UNIQUEMENT avec ce JSON
```

**Anglais :** Version équivalente en anglais pour `input_lang=en`

### Validation JSON Schema (`validate_domain_model`)

```rust
fn validate_domain_model(model: &Value) -> Result<()> {
    // 1. Charger domain_model.schema.json
    // 2. Compiler le schema avec jsonschema::Validator
    // 3. Valider le model
    // 4. Bail si erreurs
    Ok(())
}
```

**Chemins recherchés (dans l'ordre) :**
1. `../../domain_model.schema.json`
2. `../domain_model.schema.json`
3. `./domain_model.schema.json`

## Configuration

Variables d'environnement requises :

### Ollama
```bash
LLM_PROVIDER=ollama
OLLAMA_BASE_URL=http://localhost:11434  # optionnel
OLLAMA_MODEL=llama2                      # optionnel
```

### Provider Externe
```bash
LLM_PROVIDER=openai
LLM_API_KEY=sk-...
LLM_ENDPOINT=https://api.openai.com/v1/chat/completions
```

## Tests

### Test unitaire : Validation du schéma

```bash
cargo test tools::test_validate_domain_model_valid
```

Vérifie qu'un DomainModel valide passe la validation.

```bash
cargo test tools::test_validate_domain_model_invalid
```

Vérifie qu'un DomainModel invalide (mauvais type) échoue.

### Test d'intégration : Roundtrip complet

```bash
cargo test tools::normalize_terms_roundtrip -- --ignored --nocapture
```

**Nécessite :** LLM configuré (Ollama en cours ou API externe)

**Transcript de test :**
```
Un système de bibliothèque simple:
- Un Livre a un titre (obligatoire), un ISBN unique, et une date de publication
- Un Auteur a un nom obligatoire et une biographie optionnelle
- Un Livre est écrit par au moins un Auteur (1..n)
- Un Auteur peut écrire zéro ou plusieurs Livres (0..n)
- Invariant: L'ISBN doit être unique dans tout le système
```

**Vérifications :**
- ✓ Structure JSON contient `entities`, `relations`, `invariants`
- ✓ Array `entities` non vide
- ✓ Chaque entity a `id`, `name`, `attributes`
- ✓ Validation JSON Schema passe

## Usage via CLI

```bash
# Avec le MCP server
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"normalize_terms","arguments":{"input_lang":"fr","transcript":"Un Livre a un titre"}}}' | cargo run --bin mcp-server
```

## Gestion d'erreurs

**Erreurs possibles :**

1. **LLM API Error** : Ollama/API externe non disponible
   - Code : 500
   - Message : "Failed to call Ollama API" / "External API error"

2. **Parse Error** : LLM retourne du JSON invalide
   - Code : 500
   - Message : "Failed to parse LLM output as JSON"

3. **Validation Error** : JSON ne respecte pas le schéma
   - Code : 500
   - Message : "DomainModel validation failed: [details]"

4. **Schema Not Found** : domain_model.schema.json introuvable
   - Code : 500
   - Message : "Could not find domain_model.schema.json"

## Exemple de réponse réussie

**Input :**
```json
{
  "input_lang": "fr",
  "transcript": "Un Livre a un titre et un ISBN unique"
}
```

**Output :**
```json
{
  "entities": [
    {
      "id": "Livre",
      "name": "Livre",
      "attributes": [
        {"name": "titre", "type": "string", "required": true},
        {"name": "isbn", "type": "string", "required": true, "unique": true}
      ]
    }
  ],
  "relations": [],
  "invariants": []
}
```

## Performance

- **Latence typique :** 2-5 secondes (dépend du LLM)
- **Timeout :** Aucun timeout configuré (utilise timeout HTTP par défaut)
- **Cache :** Aucun cache (chaque appel est frais)

## Limitations

1. **Qualité du LLM** : La qualité du résultat dépend du modèle utilisé
2. **Langue** : Optimisé pour français et anglais
3. **Taille** : Transcripts très longs peuvent dépasser les limites de contexte
4. **Coût** : Appels API externes peuvent avoir un coût

## Prochaines améliorations

1. ✨ Ajouter retry logic en cas d'erreur temporaire
2. ✨ Cache des résultats pour transcripts identiques
3. ✨ Streaming des réponses pour transcripts longs
4. ✨ Métriques et monitoring des appels LLM
5. ✨ Support multi-langue étendu
