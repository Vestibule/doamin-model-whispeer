# Svelte 5 Migration

Ce projet utilise **Svelte 5.41.3** avec la syntaxe moderne des runes.

## Changements principaux

### 1. Réactivité avec les Runes

**Avant (Svelte 4):**
```svelte
<script>
  let count = 0;
  let name = "";
</script>
```

**Maintenant (Svelte 5):**
```svelte
<script>
  let count = $state(0);
  let name = $state("");
</script>
```

### 2. Event Handlers

**Avant (Svelte 4):**
```svelte
<form on:submit|preventDefault={handleSubmit}>
```

**Maintenant (Svelte 5):**
```svelte
<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
```

### 3. Types avec $state

Pour les valeurs typées en TypeScript :
```typescript
let result = $state<OrchestrateResult | null>(null);
let loading = $state(false);
let error = $state("");
```

### 4. API de montage du composant

**Avant (Svelte 4):**
```javascript
import App from './App.svelte';

const app = new App({
  target: document.getElementById('app'),
});
```

**Maintenant (Svelte 5):**
```javascript
import { mount } from 'svelte';
import App from './App.svelte';

const app = mount(App, {
  target: document.getElementById('app'),
});
```

## Runes disponibles dans ce projet

- **`$state()`** - Déclare un état réactif
- **`$derived()`** - Crée des valeurs dérivées (non utilisé actuellement)
- **`$effect()`** - Gère les effets de bord (non utilisé actuellement)

## Binding et directives

Les bindings fonctionnent toujours de la même manière :
```svelte
<input bind:value={name} />
<textarea bind:value={transcript} />
```

## Avantages de Svelte 5

1. **Performance améliorée** - Réactivité plus fine et efficace
2. **Type safety** - Meilleure intégration TypeScript
3. **Code plus clair** - Distinction explicite entre état réactif et variables normales
4. **Moins de magie** - Le comportement réactif est plus explicite

## Compilation

Le projet compile sans avertissements ni erreurs avec Svelte 5 :
```bash
pnpm build
# ✓ built in 240ms
```

## Compatibilité

- **Svelte**: ^5.41.3
- **@sveltejs/vite-plugin-svelte**: ^6.2.1
- **TypeScript**: ~5.6.2
- **Vite**: ^6.0.3

## Ressources

- [Svelte 5 Documentation](https://svelte.dev/docs/svelte/overview)
- [Runes Documentation](https://svelte.dev/docs/svelte/what-are-runes)
- [Migration Guide](https://svelte.dev/docs/svelte/v5-migration-guide)
