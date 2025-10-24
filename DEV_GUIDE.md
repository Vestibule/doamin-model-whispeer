# Guide de développement

## Modes de développement

Ce projet Tauri a **deux modes de développement** différents :

### 1. Mode Frontend seul (`pnpm dev`)

```bash
pnpm dev
```

**Quand l'utiliser :**
- Pour travailler uniquement sur l'interface utilisateur
- Pour tester les composants Svelte isolément
- Développement frontend rapide sans backend

**Limitations :**
- ❌ **Les commandes Tauri ne fonctionnent pas**
- ❌ Erreur: `window.__TAURI_INTERNALS__ is undefined`
- ❌ Pas d'accès au backend Rust

**URL :** http://localhost:1420/

### 2. Mode Application Tauri complète (`pnpm tauri dev`)

```bash
pnpm tauri dev
```

**Quand l'utiliser :**
- ✅ **Pour tester les commandes Tauri** (comme `orchestrate`)
- ✅ Pour tester l'intégration frontend-backend
- ✅ Pour développer et tester les fonctionnalités Rust
- ✅ Application desktop native avec hot reload

**Ce que ça lance :**
1. Compile le backend Rust
2. Lance le serveur Vite frontend
3. Ouvre une fenêtre desktop native
4. Active le hot reload pour Rust et Svelte

## Commandes disponibles

| Commande | Description | Tauri API disponible ? |
|----------|-------------|------------------------|
| `pnpm dev` | Frontend seul (Vite) | ❌ Non |
| `pnpm tauri dev` | Application complète | ✅ Oui |
| `pnpm build` | Build frontend uniquement | - |
| `pnpm tauri build` | Build application complète | - |

## Résolution des erreurs Tauri

### `window.__TAURI_INTERNALS__ is undefined`

Cette erreur apparaît quand vous essayez d'utiliser les API Tauri (comme `invoke()`, `listen()`) dans un navigateur web standard.

**Solution :** Utilisez `pnpm tauri dev` au lieu de `pnpm dev`

### `can't access property "transformCallback", window.__TAURI_INTERNALS__ is undefined`

Variante de la même erreur, spécifiquement pour les événements Tauri (`listen()`).

**Solution :** Lancez l'application avec `pnpm tauri dev`

Le composant `SpeechInput` détecte automatiquement s'il est dans Tauri et affiche un message d'erreur approprié si ce n'est pas le cas.

## Variables d'environnement requises

Pour que la commande `orchestrate` fonctionne, configurez ces variables :

```bash
# .env
LLM_PROVIDER=ollama  # ou "external"
OLLAMA_BASE_URL=http://localhost:11434
OLLAMA_MODEL=llama3

# Si vous utilisez un provider externe :
# LLM_API_KEY=your_api_key
# LLM_ENDPOINT=https://api.openai.com/v1/chat/completions

MCP_SERVER_PATH=../mcp/mcp-server/target/release/mcp-server
```

## Architecture

```
┌─────────────────────────────────────┐
│   Frontend (Svelte 5 + TypeScript)  │
│   - App.svelte                       │
│   - src/lib/tauri.ts                 │
└──────────────┬──────────────────────┘
               │ invoke("orchestrate")
               ▼
┌─────────────────────────────────────┐
│   Backend (Rust + Tauri)             │
│   - orchestrate command              │
│   - llm_integration                  │
│   - mcp_client                       │
└─────────────────────────────────────┘
```

## Hot Reload

- **Frontend (Svelte)** : Rechargement automatique instantané
- **Backend (Rust)** : Recompilation et redémarrage automatique (quelques secondes)

## Ports utilisés

- **Frontend Vite** : 1420
- **HMR (Hot Module Replacement)** : 1421

## Workflow recommandé

1. **Développement UI uniquement** → `pnpm dev`
2. **Test des fonctionnalités Tauri** → `pnpm tauri dev`
3. **Build de production** → `pnpm tauri build`

## Debugging

Si `pnpm tauri dev` ne démarre pas :

1. Vérifiez qu'aucun processus n'utilise le port 1420 :
   ```bash
   lsof -i :1420
   ```

2. Tuez le processus si nécessaire :
   ```bash
   lsof -ti:1420 | xargs kill -9
   ```

3. Relancez `pnpm tauri dev`
