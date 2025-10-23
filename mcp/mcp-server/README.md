# Domain Model Note-Taking - MCP Server

Serveur MCP (Model Context Protocol) pour la génération et validation de modèles de domaine à partir de langage naturel.

## 📋 Table des Matières

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Utilisation](#utilisation)
- [Développement](#développement)
- [Tests](#tests)
- [Documentation](#documentation)

## 🚀 Installation

### Prérequis

- Rust 1.70+ (installation via [rustup](https://rustup.rs/))
- [Task](https://taskfile.dev/) pour l'automatisation (installation : `brew install go-task`)
- Python 3 (pour pretty-print JSON)

### Build

```bash
# Tout construire
task build

# Ou séparer les builds
task build:server  # MCP server seulement
task build:cli     # CLI tool seulement
```

## ⚡ Quick Start

### Lister les tâches disponibles

```bash
task
# ou
task --list
```

### Démarrage rapide

```bash
# Build + run CLI avec sample
task quick:start

# Démo complète (MCP tools + CLI)
task quick:demo

# Tests rapides
task quick:test
```

### Exemples de base

```bash
# Lister les outils MCP
task mcp:list

# Exécuter le CLI
task cli:run

# Avec tracing
task cli:trace

# Pipeline complet
task cli:full
```

## 🏗️ Architecture

### Deux binaires distincts

1. **mcp-server** : Serveur JSON-RPC pour intégration MCP
   - Communication via stdin/stdout
   - Protocole JSON-RPC 2.0
   - 5 outils exposés

2. **mcp-cli** : Outil en ligne de commande
   - Arguments CLI classiques
   - Pipeline complet : transcript → model → markdown + mermaid
   - Support retry, tracing, validation

### Outils MCP disponibles

| Outil | Description |
|-------|-------------|
| `generate_domain_model` | Génère un DomainModel complet depuis du langage naturel |
| `normalize_terms` | Extrait le modèle depuis une transcription |
| `emit_markdown` | Génère la documentation Markdown structurée |
| `emit_mermaid` | Génère les diagrammes Mermaid (ER ou class) |
| `validate_model` | Valide la cohérence et la complétude du modèle |

## 📖 Utilisation

### MCP Server

Le serveur MCP suit le protocole JSON-RPC :

```bash
# Lister les tools
task mcp:list

# Initialiser le serveur
task mcp:init

# Tester normalize_terms
task mcp:normalize

# Tester validation
task mcp:validate
```

**Utilisation manuelle :**

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | \
  ./target/release/mcp-server | python3 -m json.tool
```

Voir [README_MCP.md](README_MCP.md) pour plus de détails.

### CLI Tool

Le CLI offre un workflow complet :

```bash
# Run basique
task cli:run

# Avec tracing (logs JSON structurés)
task cli:trace

# Avec retry (répare JSON invalide)
task cli:retry

# Pipeline complet
task cli:full
```

**Options CLI disponibles :**

```bash
./target/release/mcp-cli \
  --input samples/voice.json \           # Fichier transcript (JSON Lines)
  --emit-md artifacts/spec.md \          # Génère Markdown
  --emit-mmd artifacts/model.mmd \       # Génère Mermaid
  --retry 2 \                            # Nombre de retries (défaut: 2)
  --trace \                              # Active tracing JSON
  --dry-run-llm \                        # Mock LLM (pas d'API call)
  --validate-only                        # Valide sans émettre fichiers
```

## 🧪 Tests

### Toutes les catégories de tests

```bash
# Tous les tests
task test

# Tests unitaires
task test:unit

# Tests des tools (emit_markdown, emit_mermaid)
task test:tools

# Tests d'orchestration (pipeline)
task test:orchestration

# Test d'idempotence (Model → Markdown → Model)
task test:idempotence
```

### Tests spécifiques

```bash
# Test des symboles ER Mermaid
cargo test tools::emit_mermaid_er_symbols -- --nocapture

# Test des sections Markdown
cargo test tools::emit_markdown_sections -- --nocapture

# Test d'idempotence
cargo test orchestration::idempotence -- --nocapture
```

## 🛠️ Développement

### Workflow de développement

```bash
# Vérifier avant commit (format, lint, test)
task dev:check

# Auto-fix des problèmes
task dev:fix

# Format du code
task fmt

# Vérifier format sans modifier
task fmt:check

# Linter
task lint
```

### Build en mode dev

```bash
# Build rapide (non optimisé)
task build:dev

# Watch mode (avec cargo-watch)
cargo watch -x 'build --bin mcp-cli'
```

### Tracing et debugging

```bash
# Activer tracing JSON
export RUST_LOG=info,domain=debug
task cli:trace

# Logs disponibles :
# - domain::cli   : Événements CLI
# - domain::llm   : Interactions LLM (prompts hashés, jamais de PII)
```

### Variables d'environnement

```bash
# LLM Provider (défaut: ollama)
export LLM_PROVIDER=ollama
export OLLAMA_BASE_URL=http://localhost:11434
export OLLAMA_MODEL=llama2

# Ou provider externe
export LLM_PROVIDER=openai
export LLM_API_KEY=your_api_key
export LLM_ENDPOINT=https://api.openai.com/v1/chat/completions

# Niveau de log
export RUST_LOG=info,domain=debug
```

## 📚 Documentation

### Générer la documentation Rust

```bash
task doc
```

### Lire les READMEs

```bash
task doc:readme
```

### Documentation disponible

- **README.md** (ce fichier) : Vue d'ensemble et Task
- **README_MCP.md** : Protocole MCP et intégration
- **test_mcp.sh** : Script legacy (remplacé par Task)
- **Cargo docs** : `task doc`

## 🧹 Nettoyage

```bash
# Tout nettoyer
task clean

# Artifacts seulement
task clean:artifacts
```

## 🎯 Cas d'Usage

### 1. Développement local

```bash
# 1. Construire
task build

# 2. Tester
task test

# 3. Essayer avec sample
task cli:run

# 4. Vérifier artifacts
ls -lh artifacts/
```

### 2. Intégration MCP (Warp, Claude)

```json
{
  "mcpServers": {
    "domain-model": {
      "command": "/path/to/mcp-server/target/release/mcp-server",
      "args": []
    }
  }
}
```

Puis :
```bash
task mcp:list
```

### 3. CI/CD

```bash
# Pre-commit checks
task dev:check

# Build release
task build

# Run tests
task test
```

## 🔍 Fonctionnalités

### Pipeline complet

```
Transcript (JSON Lines)
    ↓
normalize_terms (LLM)
    ↓
validate_model
    ↓
├─ emit_markdown
└─ emit_mermaid
```

### Retry avec repair

Si le LLM retourne du JSON invalide :
1. Parse échoue
2. Si retries restantes : envoie au LLM pour repair
3. Repair prompt corrige structure sans changer contenu
4. Retry parsing

### Tracing privacy-first

- Prompts hashés (SHA256)
- Jamais de PII en clair
- Logs JSON structurés
- Métadonnées : hash, taille, durée

### Idempotence testing

Test vérifie stabilité du cycle :
```
Model → emit_markdown → parse_markdown → Model'
```

Compare avec diff structuré.

## 📝 Exemples de Tasks

### Build

```bash
task build              # Release build
task build:server       # Server seulement
task build:cli          # CLI seulement
task build:dev          # Dev build (rapide)
```

### Test

```bash
task test               # Tous
task test:unit          # Unitaires
task test:tools         # Tools (emit_*)
task test:orchestration # Pipeline
task test:idempotence   # Idempotence
```

### MCP

```bash
task mcp:list           # Liste tools
task mcp:init           # Initialize
task mcp:normalize      # Test normalize
task mcp:validate       # Test validate
```

### CLI

```bash
task cli:run            # Basique
task cli:trace          # Avec tracing
task cli:retry          # Avec retry
task cli:full           # Complet
```

### Dev

```bash
task dev:check          # Pre-commit
task dev:fix            # Auto-fix
task fmt                # Format
task lint               # Lint
```

### Quick

```bash
task quick:start        # Build + run
task quick:test         # Tests
task quick:demo         # Démo complète
```

## 🤝 Contribution

1. Fork le repo
2. Créer une branche feature
3. `task dev:check` avant commit
4. Ouvrir une PR

## 📄 License

Voir LICENSE dans le repo racine.

## 🔗 Liens

- [Task documentation](https://taskfile.dev/)
- [MCP Protocol](https://modelcontextprotocol.io/)
- [Rust Book](https://doc.rust-lang.org/book/)
