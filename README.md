# Domain Model Note Taking

Application Tauri pour la prise de notes avec génération automatique de modèles de domaine.

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

## Documentation

- **[DEV_GUIDE.md](./DEV_GUIDE.md)** - Guide de développement et résolution des erreurs
- **[SVELTE5.md](./SVELTE5.md)** - Migration et syntaxe Svelte 5
- **[ORCHESTRATE.md](./ORCHESTRATE.md)** - Documentation de la commande `orchestrate`
- **[SPEECH_TO_TEXT.md](./SPEECH_TO_TEXT.md)** - Intégration reconnaissance vocale
- **[BACKEND_RECORDING.md](./BACKEND_RECORDING.md)** - Enregistrement audio backend avec VAD
- **[WARP.md](./WARP.md)** - Configuration du projet pour Warp

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
