# Speech-to-Text Integration

Ce projet intègre la reconnaissance vocale pour permettre la dictée directe dans l'interface.

## Solution implémentée : Backend Rust avec VAD

### Fonctionnalités

- ✅ **Enregistrement côté backend** : Le microphone est géré par Rust, pas le navigateur
- ✅ **Détection d'activité vocale (VAD)** : webrtc-vad détecte automatiquement quand vous parlez
- ✅ **Segmentation automatique** : Découpe l'audio en utterances basées sur les silences
- ✅ **Transcription Whisper (prête)** : Infrastructure prête pour transcription IA
- ✅ **Interface intuitive** : Bouton microphone avec états visuels (🎤 → 🔴 → ⏳)

### Utilisation

1. Lancez l'application avec `pnpm tauri dev`
2. Cliquez sur le bouton microphone 🎤 à droite du textarea
3. Autorisez l'accès au microphone si demandé (permissions système)
4. Parlez normalement - le bouton devient rouge 🔴
5. Cliquez à nouveau pour arrêter - le bouton devient sablier ⏳
6. La transcription apparaît automatiquement dans le textarea

### Composant Svelte

Le composant `SpeechInput.svelte` est réutilisable :

```svelte
<script>
  import SpeechInput from './lib/SpeechInput.svelte';
  
  let transcript = $state("");
</script>

<SpeechInput bind:value={transcript} />
```

### Props disponibles

| Prop | Type | Description | Défaut |
|------|------|-------------|--------|
| `value` | `string` (bindable) | Texte transcrit | `""` |
| `onTranscript` | `(text: string) => void` | Callback appelé à chaque transcription | - |
| `placeholder` | `string` | Texte placeholder | `"Click microphone to speak..."` |

### Compatibilité navigateur

L'API Web Speech fonctionne sur :
- ✅ Chrome/Edge (Chromium)
- ✅ Safari (macOS/iOS)
- ❌ Firefox (support limité)

Si le navigateur ne supporte pas l'API, un message d'avertissement s'affiche.

## Solution alternative : Whisper (Backend)

### État actuel

Le backend Rust contient du code pour Whisper.cpp mais nécessite :

1. **Modèle Whisper** : Télécharger un modèle (ggml-base.en.bin)
2. **Correction API** : L'API whisper-rs a changé depuis la version utilisée
3. **Fichiers audio** : Transcription à partir de fichiers WAV

### Commandes Tauri disponibles

```typescript
// Démarrer l'enregistrement avec détection de la parole
await startRecording();

// Transcrire un fichier audio (nécessite configuration)
const result = await transcribeAudio("/path/to/audio.wav");
// { text: string, language: string | null, duration_ms: number }
```

### Configuration requise (si Whisper est utilisé)

```bash
# .env
WHISPER_MODEL_PATH=models/ggml-base.en.bin
```

### Télécharger un modèle Whisper

```bash
mkdir -p models
cd models

# Modèle base en anglais (~140 MB)
curl -LO https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin

# Autres options :
# tiny.en (~75 MB) - plus rapide, moins précis
# small.en (~466 MB) - bon compromis
# medium.en (~1.5 GB) - très précis
```

## Workflow recommandé

Pour la plupart des cas d'usage, **utilisez l'API Web Speech** :
- Pas de setup requis
- Fonctionne directement dans le navigateur
- Performance excellente
- Gratuit

Utilisez Whisper seulement si vous avez besoin de :
- Transcription hors ligne
- Support de langues spécifiques
- Contrôle total sur le modèle

## Architecture

```
┌─────────────────────────────────┐
│   SpeechInput.svelte            │
│   (Web Speech API)              │
└────────────┬────────────────────┘
             │ bind:value
             ▼
┌─────────────────────────────────┐
│   App.svelte                    │
│   transcript textarea           │
└────────────┬────────────────────┘
             │ invoke("orchestrate")
             ▼
┌─────────────────────────────────┐
│   Backend Rust                  │
│   (LLM + MCP)                   │
└─────────────────────────────────┘
```

## Développement futur

Pour activer Whisper :

1. Corriger l'API whisper-rs dans `speech_to_text.rs`
2. Télécharger un modèle Whisper
3. Configurer `WHISPER_MODEL_PATH`
4. Créer une UI pour choisir entre Web Speech et Whisper

## Exemple complet

```svelte
<script lang="ts">
  import { orchestrate } from './lib/tauri';
  import SpeechInput from './lib/SpeechInput.svelte';

  let transcript = $state("");
  let result = $state(null);

  async function handleOrchestrate() {
    result = await orchestrate(transcript);
  }
</script>

<div class="container">
  <div class="input-group">
    <textarea bind:value={transcript} />
    <SpeechInput bind:value={transcript} />
  </div>
  
  <button onclick={handleOrchestrate}>
    Process
  </button>

  {#if result}
    <pre>{result.markdown}</pre>
  {/if}
</div>
```

## Troubleshooting

### "Microphone access denied"
- Vérifiez les permissions du navigateur
- Sur macOS : Préférences Système → Sécurité → Microphone

### "Speech recognition not supported"
- Utilisez Chrome ou Safari
- Assurez-vous d'avoir une connexion internet (Web Speech nécessite une connexion)

### Le texte ne s'affiche pas
- Parlez clairement et à un volume normal
- Vérifiez que le microphone fonctionne dans d'autres applications
- Essayez de redémarrer le navigateur
