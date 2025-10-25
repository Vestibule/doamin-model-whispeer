# Domain Model Note Taking

Application Tauri pour créer des Domain Model Canvas (DDD) via une **interview guidée** avec traitement LLM en temps réel.

## 🎯 Fonctionnalités principales

### Mode Interview Guidée ✨ (Nouveau)
- **9 sections structurées** basées sur Domain-Driven Design
- **Traitement LLM automatique** : vos réponses sont transformées en documentation professionnelle
- **Aperçu temps réel** du canvas pendant l'interview
- **Support audio & texte** pour répondre aux questions
- **Navigation flexible** entre les sections
- **Canvas complet généré** en markdown formaté

📚 **[Guide d'utilisation complet](./INTERVIEW_USAGE.md)**

### Mode Transcript Libre (Original)
- Capture audio avec voice activity detection
- Transcription via Whisper
- Génération de domain model via LLM

## Stack technique

- **Frontend**: Svelte 5 (avec runes) + TypeScript
- **Backend**: Rust + Tauri 2
- **Build**: Vite 6
- **Package Manager**: pnpm

## Démarrage rapide

### Développement

```bash
# Frontend seul (pas d'accès aux commandes Tauri)
pnpm dev

# Application complète (recommandé)
pnpm tauri dev
```

⚠️ **Important**: Pour utiliser la commande `orchestrate` et les autres fonctionnalités Tauri, vous devez utiliser `pnpm tauri dev`

### Build de production

```bash
pnpm tauri build
```

## Configuration LLM

L'application nécessite un LLM configuré. Créez un fichier `.env` à la racine :

### Option 1: Ollama (Local)
```env
LLM_PROVIDER=ollama
OLLAMA_BASE_URL=http://localhost:11434
OLLAMA_MODEL=domain-model-mistral
```

### Option 2: LLM Externe (OpenAI-compatible)
```env
LLM_PROVIDER=external
LLM_API_KEY=votre_clé_api
LLM_ENDPOINT=https://api.openai.com/v1/chat/completions
```

## Documentation

### Interview guidée (Nouveau)
- **[INTERVIEW_USAGE.md](./INTERVIEW_USAGE.md)** - 🔥 **Guide utilisateur complet**
- **[INTERVIEW_FEATURE.md](./INTERVIEW_FEATURE.md)** - Documentation technique de la fonctionnalité
- **[IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md)** - Résumé de l'implémentation

### Développement
- **[DEV_GUIDE.md](./DEV_GUIDE.md)** - Guide de développement et résolution des erreurs
- **[SVELTE5.md](./SVELTE5.md)** - Migration et syntaxe Svelte 5
- **[ORCHESTRATE.md](./ORCHESTRATE.md)** - Documentation de la commande `orchestrate`
- **[SPEECH_TO_TEXT.md](./SPEECH_TO_TEXT.md)** - Intégration reconnaissance vocale
- **[BACKEND_RECORDING.md](./BACKEND_RECORDING.md)** - Enregistrement audio backend avec VAD
- **[WARP.md](./WARP.md)** - Configuration du projet pour Warp

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
