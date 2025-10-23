# Implémentation LLM Router + MCP Integration

## Vue d'ensemble

Ce projet intègre un routeur LLM qui génère des DomainModel JSON stricts à partir de transcripts en langage naturel. Le LLM est contraint par un system prompt strict pour ne produire **que** du JSON conforme au schéma DomainModel.

## Architecture

```
┌─────────────────┐
│  CLI / Tauri    │
└────────┬────────┘
         │
         v
┌─────────────────┐
│   MCP Server    │ ← generate_domain_model tool
│  (mcp-server)   │
└────────┬────────┘
         │
         v
┌─────────────────┐
│   LLM Router    │ ← Routes to Ollama or External
│  (llm_router)   │
└────────┬────────┘
         │
    ┌────┴────┐
    v         v
┌────────┐ ┌──────────┐
│ Ollama │ │ External │
│ (local)│ │ Provider │
└────────┘ └──────────┘
```

## Composants

### 1. LLM Router (`src-tauri/src/llm_router.rs`)

**Responsabilités :**
- Détecte le provider LLM depuis les variables d'environnement
- Route les requêtes vers Ollama (`POST /api/generate`) ou provider externe
- Parse les réponses JSON en `DomainModelResponse`

**Configuration via `.env` :**
```bash
# Ollama
LLM_PROVIDER=ollama
OLLAMA_BASE_URL=http://localhost:11434
OLLAMA_MODEL=llama2

# Ou External Provider
LLM_PROVIDER=openai
LLM_API_KEY=sk-...
LLM_ENDPOINT=https://api.openai.com/v1/chat/completions
```

**API principale :**
```rust
pub async fn generate_domain_model(
    &self,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<DomainModelResponse>
```

### 2. MCP Server (`mcp/mcp-server/src/main.rs`)

**Outil ajouté :**
```json
{
  "name": "generate_domain_model",
  "description": "Generate a complete DomainModel from natural language using LLM",
  "inputSchema": {
    "type": "object",
    "properties": {
      "transcript": {"type": "string"},
      "input_lang": {"type": "string", "default": "fr"}
    },
    "required": ["transcript"]
  }
}
```

**Handler :** Actuellement un placeholder, prêt pour intégration complète du LLM router.

### 3. CLI Tool (`mcp/mcp-server/src/cli.rs`)

**Commande :**
```bash
cargo run --bin mcp-cli -- [--dry-run-llm] --input samples/transcript.jsonl
```

**Fonctionnalités :**
- Lit un fichier JSONL de transcript
- Appelle le LLM (ou simule en dry-run)
- Affiche le DomainModel JSON généré

**Exemple de sortie :**
```
📝 Transcript loaded (8 lines)
🤖 Mode: DRY-RUN

⏳ Generating DomainModel from transcript...
✅ DomainModel generated successfully!

{
  "entities": [...],
  "relations": [...],
  "invariants": [...]
}
```

### 4. System Prompt Strict

Le LLM reçoit ce prompt système :

```
Tu es un normalizer de Domain Model. Rends UNIQUEMENT un JSON valide 
DomainModel conforme au schema. Interdis les champs non listés.

Schema DomainModel (STRICT - aucun champ supplémentaire autorisé):
{
  "entities": [...],
  "relations": [...],
  "invariants": [...]
}

RÈGLES STRICTES:
1. AUCUN champ en dehors de ce schema
2. Tous les champs "obligatoire" DOIVENT être présents
3. Les types enum DOIVENT correspondre exactement
4. Les patterns regex DOIVENT être respectés
5. Réponds UNIQUEMENT avec ce JSON, pas de tool_calls
```

## Schéma DomainModel

Voir `mcp/domain_model.schema.json` pour le schéma complet.

**Résumé des types principaux :**

### Entity
```json
{
  "id": "string",
  "name": "string",
  "description": "string (optional)",
  "attributes": [
    {
      "name": "string",
      "type": "string|number|integer|boolean|date|datetime|email|url|uuid|json|text",
      "required": boolean,
      "unique": boolean
    }
  ],
  "primaryKey": ["string"],
  "uniqueConstraints": [...]
}
```

### Relation
```json
{
  "id": "string",
  "name": "string",
  "from": {"entityId": "string"},
  "to": {"entityId": "string"},
  "cardinality": {
    "from": "0..1|1|0..n|1..n|*",
    "to": "0..1|1|0..n|1..n|*"
  }
}
```

### Invariant
```json
{
  "id": "string",
  "name": "string",
  "type": "uniqueness|referential_integrity|domain_constraint|cardinality|business_rule|temporal|aggregation",
  "expression": "string",
  "severity": "error|warning|info"
}
```

## Utilisation

### 1. Configuration

```bash
# Dans src-tauri/ ou mcp/mcp-server/
cp .env.example .env
# Éditer .env avec vos credentials
```

### 2. Test avec le CLI

```bash
cd mcp/mcp-server

# Mode dry-run (simulation)
cargo run --bin mcp-cli -- --dry-run-llm --input ../../samples/transcript.jsonl

# Mode live (LLM réel)
cargo run --bin mcp-cli -- --input ../../samples/transcript.jsonl
```

### 3. Intégration Tauri

```rust
use domain_model_note_taking_lib::llm_integration::LlmIntegration;

let integration = LlmIntegration::new()?;
let domain_model = integration
    .process_request("User entity has email and password")
    .await?;
```

## Format des fichiers

### Transcript JSONL (`samples/transcript.jsonl`)

```jsonl
{"speaker": "user", "text": "Je veux modéliser un système de bibliothèque"}
{"speaker": "user", "text": "Un Livre a un titre, un ISBN unique"}
{"speaker": "user", "text": "Un Auteur a un nom et une biographie"}
```

## Tests

```bash
# Test du CLI en dry-run
cd mcp/mcp-server
cargo run --bin mcp-cli -- --dry-run-llm --input ../../samples/transcript.jsonl

# Build release
cargo build --release --bin mcp-cli

# Compilation Tauri
cd src-tauri
cargo check --lib
```

## Sécurité

**IMPORTANT :** Ne jamais committer les fichiers `.env` !

Les `.gitignore` ont été mis à jour pour exclure :
```
.env
```

Utilisez `.env.example` comme template pour d'autres développeurs.

## Prochaines étapes

1. **Implémenter l'intégration complète** du LLM router dans le handler MCP `generate_domain_model`
2. **Ajouter la validation** du DomainModel généré contre le JSON schema
3. **Créer des commandes Tauri** pour exposer `generate_domain_model` à l'UI Vue
4. **Ajouter des tests d'intégration** avec différents providers LLM
5. **Optimiser le system prompt** basé sur les résultats réels

## Documentation

- **LLM Router :** `src-tauri/LLM_ROUTER.md`
- **CLI Tool :** `mcp/mcp-server/CLI.md`
- **Schéma DomainModel :** `mcp/domain_model.schema.json`
- **WARP Rules :** `WARP.md`

## Dépendances ajoutées

**`src-tauri/Cargo.toml` :**
```toml
reqwest = { version = "0.11", features = ["json"] }
dotenvy = "0.15"
```

**`mcp/mcp-server/Cargo.toml` :**
```toml
clap = { version = "4", features = ["derive"] }
dotenvy = "0.15"
reqwest = { version = "0.11", features = ["json"] }
```

## Principe clé

> **Le LLM ne parle jamais à l'UI. Il n'émet que des structures pour les tools.**

Le system prompt contraint strictement le LLM à ne produire que du JSON DomainModel valide, sans aucun texte libre ni champs supplémentaires.
