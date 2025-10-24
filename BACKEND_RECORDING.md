# Backend Audio Recording avec Whisper

## Architecture

```
┌────────────────────────────────┐
│   Frontend (Svelte)            │
│   - SpeechInput component      │
│   - Start/Stop buttons         │
└───────────┬────────────────────┘
            │ invoke() commands
            ▼
┌────────────────────────────────┐
│   RecordingManager (Rust)      │
│   - State management           │
│   - Event emission             │
└───────────┬────────────────────┘
            │
      ┌─────┴─────┐
      ▼           ▼
┌──────────┐  ┌──────────────┐
│  Audio   │  │  Whisper     │
│  Session │  │  STT         │
│  (VAD)   │  │  (stub)      │
└──────────┘  └──────────────┘
```

## Flux de données

### 1. Démarrage de l'enregistrement

```
User clicks 🎤
  → startRecording()
  → RecordingManager.start_recording()
  → AudioSession.start_recording()
  → VAD loop starts
  → Emit "recording-state-changed": "recording"
```

### 2. Pendant l'enregistrement

```
Audio stream
  → webrtc-vad detects voice activity
  → Segments saved as WAV files
  → Files: /tmp/domain-model-audio/utterance_XXXX.wav
```

### 3. Arrêt et transcription

```
User clicks 🔴
  → stopRecording()
  → Recording stops
  → Emit "recording-state-changed": "processing"
  → For each utterance:
      → SpeechToText.transcribe_file()
      → Emit "transcription-result": { text, language, duration_ms }
  → Emit "recording-state-changed": "idle"
```

## Configuration

### Variables d'environnement

```bash
# .env (optionnel)
WHISPER_MODEL_PATH=models/ggml-base.en.bin
```

### Télécharger un modèle Whisper

```bash
mkdir -p models
cd models

# Modèle tiny (75 MB) - rapide
curl -LO https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin

# Modèle base (140 MB) - recommandé
curl -LO https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin

# Modèle small (466 MB) - précis
curl -LO https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin
```

## Paramètres VAD

Dans `audio_session.rs`, les paramètres par défaut sont :

```rust
AudioSessionConfig {
    silence_duration_ms: 1000,        // 1s de silence pour terminer une utterance
    min_utterance_duration_ms: 300,   // Utterance minimale de 300ms
    vad_mode: VadMode::Aggressive,    // Détection agressive
    output_dir: /tmp/domain-model-audio
}
```

### Ajuster la sensibilité

Pour ajuster la détection de parole, modifiez dans `lib.rs` :

```rust
let config = AudioSessionConfig {
    silence_duration_ms: 1500,  // Plus de tolérance au silence
    min_utterance_duration_ms: 500,  // Utterances plus longues
    vad_mode: VadMode::Quality,  // Moins agressif
    output_dir: output_dir.clone(),
};
```

## Events Tauri

Le backend émet plusieurs événements vers le frontend :

### `recording-state-changed`
```typescript
listen<string>('recording-state-changed', (event) => {
  // event.payload: "idle" | "recording" | "processing"
});
```

### `transcription-result`
```typescript
listen<TranscriptionResult>('transcription-result', (event) => {
  const { text, language, duration_ms } = event.payload;
});
```

### `transcription-error`
```typescript
listen<string>('transcription-error', (event) => {
  console.error('Transcription failed:', event.payload);
});
```

### `recording-error`
```typescript
listen<string>('recording-error', (event) => {
  console.error('Recording failed:', event.payload);
});
```

## Commandes Tauri

### `start_recording`
```typescript
import { startRecording } from './lib/tauri';

const status = await startRecording();
// Returns: "Recording started. Audio will be saved to: /tmp/domain-model-audio"
```

### `stop_recording`
```typescript
import { stopRecording } from './lib/tauri';

const status = await stopRecording();
// Returns: "Recording stopped. Processing utterances..."
```

### `transcribe_audio`
```typescript
import { transcribeAudio } from './lib/tauri';

const result = await transcribeAudio('/path/to/audio.wav');
// Returns: { text: string, language: string | null, duration_ms: number }
```

## État actuel de Whisper

### ⚠️ Status: Stub Implementation

La transcription Whisper est actuellement en mode stub. Les fichiers audio sont enregistrés correctement, mais la transcription retourne :

```
[Transcription stub: audio file XXXX bytes - configure WHISPER_MODEL_PATH to enable real transcription]
```

### Pour activer Whisper

1. **Télécharger un modèle** (voir section ci-dessus)

2. **Fixer l'API whisper-rs** dans `speech_to_text.rs`:
   - Remplacer le stub par l'implémentation réelle
   - Utiliser les méthodes correctes de l'API whisper-rs 0.15.1

3. **Tester**:
   ```bash
   WHISPER_MODEL_PATH=models/ggml-base.en.bin pnpm tauri dev
   ```

## Debugging

### Vérifier les fichiers audio

```bash
ls -lh /tmp/domain-model-audio/
```

Les fichiers WAV devraient être :
- Format: 16-bit PCM mono
- Sample rate: 16000 Hz
- Nommés: `utterance_0001.wav`, `utterance_0002.wav`, etc.

### Logs backend

Les logs Rust apparaissent dans la console Tauri :

```
[INFO] Loading Whisper model from models/ggml-base.en.bin
[INFO] Starting audio recording thread
[INFO] Found 3 utterances to transcribe
[INFO] Transcribing utterance 1: /tmp/.../utterance_0001.wav
[WARN] Transcribing /tmp/.../utterance_0001.wav (stub - Whisper not yet configured)
```

### Tester l'audio manuellement

```bash
# Installer ffplay (part of ffmpeg)
brew install ffmpeg

# Écouter un fichier enregistré
ffplay /tmp/domain-model-audio/utterance_0001.wav
```

## Performance

### Latence attendue

- **Détection VAD**: < 30ms
- **Sauvegarde WAV**: < 10ms
- **Transcription Whisper** (avec modèle):
  - tiny: ~100-200ms par utterance
  - base: ~200-500ms par utterance
  - small: ~500ms-1s par utterance

### Optimisations futures

1. **Transcription en streaming** : Transcrire pendant l'enregistrement
2. **Modèle en cache** : Précharger le modèle Whisper au démarrage
3. **GPU acceleration** : Utiliser CUDA/Metal si disponible
4. **Quantification** : Utiliser des modèles quantifiés (Q5, Q8)

## Limitations actuelles

1. **Pas de pause/resume** : L'enregistrement doit être arrêté puis redémarré
2. **Pas de feedback audio** : Pas d'indicateur du niveau sonore
3. **Pas de sélection de langue** : Hardcodé à "en" (anglais)
4. **Whisper en stub** : Nécessite configuration du modèle

## Roadmap

- [ ] Implémenter vraie transcription Whisper
- [ ] Ajouter feedback niveau audio visuel
- [ ] Support multilingue (auto-détection)
- [ ] Transcription en streaming
- [ ] Export des utterances en JSON
- [ ] Intégration directe avec `orchestrate()`
