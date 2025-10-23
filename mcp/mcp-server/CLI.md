# MCP CLI Tool

CLI pour tester l'outil MCP `generate_domain_model` avec intégration LLM.

## Installation

```bash
cd mcp/mcp-server
cargo build --release --bin mcp-cli
```

## Configuration

Créez un fichier `.env` dans `mcp/mcp-server/` :

```bash
cp .env.example .env
# Éditez .env avec vos credentials
```

## Utilisation

### Mode Dry-Run (sans appel LLM réel)

Pour tester rapidement avec une réponse simulée :

```bash
cargo run --bin mcp-cli -- --dry-run-llm --input ../../samples/transcript.jsonl
```

Ou avec le binaire compilé :

```bash
./target/release/mcp-cli --dry-run-llm --input ../../samples/transcript.jsonl
```

### Mode Live (avec LLM réel)

Assurez-vous que votre `.env` est configuré, puis :

```bash
cargo run --bin mcp-cli -- --input ../../samples/transcript.jsonl
```

## Format du fichier d'entrée

Le fichier d'entrée doit être au format JSONL (JSON Lines) :

```jsonl
{"speaker": "user", "text": "Je veux modéliser un système de bibliothèque"}
{"speaker": "user", "text": "Un Livre a un titre, un ISBN unique, et une date de publication"}
{"speaker": "user", "text": "Un Auteur a un nom et une biographie optionnelle"}
```

Chaque ligne est un objet JSON avec :
- `speaker` : identifiant du locuteur (peut être ignoré)
- `text` : le texte de la ligne de transcript

## Sortie

Le CLI génère un DomainModel JSON conforme au schéma avec :

- **entities** : liste des entités du domaine
- **relations** : liste des relations entre entités
- **invariants** : liste des invariants métier

Exemple :

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
  "relations": [...],
  "invariants": [...]
}
```

## Providers LLM supportés

### Ollama (Local)

```bash
LLM_PROVIDER=ollama
OLLAMA_BASE_URL=http://localhost:11434
OLLAMA_MODEL=llama2
```

### OpenAI

```bash
LLM_PROVIDER=openai
LLM_API_KEY=sk-...
LLM_ENDPOINT=https://api.openai.com/v1/chat/completions
```

### Anthropic

```bash
LLM_PROVIDER=anthropic
LLM_API_KEY=sk-ant-...
LLM_ENDPOINT=https://api.anthropic.com/v1/messages
```

## Exemples

### Exemple 1 : Mode dry-run

```bash
$ cargo run --bin mcp-cli -- --dry-run-llm --input ../../samples/transcript.jsonl

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

### Exemple 2 : Avec Ollama local

```bash
# S'assurer qu'Ollama tourne
$ ollama serve

# Dans un autre terminal
$ cargo run --bin mcp-cli -- --input ../../samples/transcript.jsonl

📝 Transcript loaded (8 lines)
🤖 Mode: LIVE LLM

⏳ Generating DomainModel from transcript...
✅ DomainModel generated successfully!
...
```

## Intégration MCP

L'outil `generate_domain_model` est également disponible comme outil MCP dans le serveur.

Définition de l'outil :

```json
{
  "name": "generate_domain_model",
  "description": "Generate a complete DomainModel from natural language using LLM",
  "inputSchema": {
    "type": "object",
    "properties": {
      "transcript": {
        "type": "string",
        "description": "Natural language transcript describing the domain model"
      },
      "input_lang": {
        "type": "string",
        "description": "Input language code (e.g., 'en', 'fr')",
        "default": "fr"
      }
    },
    "required": ["transcript"]
  }
}
```

## Contraintes du LLM

Le LLM est contraint par un system prompt strict qui impose :

1. **Aucun champ** en dehors du schéma DomainModel
2. **Tous les champs obligatoires** doivent être présents
3. **Les types enum** doivent correspondre exactement
4. **Les patterns regex** doivent être respectés
5. **Réponse JSON uniquement**, pas de texte libre

Cela garantit que la sortie est toujours un DomainModel valide.
