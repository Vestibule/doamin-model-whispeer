# Audio Streaming CLI

Interface en ligne de commande pour le streaming audio avec détection d'activité vocale (VAD).

## Utilisation

```bash
RUST_LOG=info cargo run -- --stream --vad-threshold 0.6 --max-chunk-ms 2000
```

## Options

### `--stream`
Active le mode streaming audio avec détection VAD.

### `--vad-threshold <THRESHOLD>` 
Seuil de sensibilité du VAD (0.0 à 1.0).
- **0.0 - 0.25**: Mode "Quality" (moins agressif, conserve plus d'audio)
- **0.25 - 0.5**: Mode "LowBitrate" (équilibré)
- **0.5 - 0.75**: Mode "Aggressive" (plus agressif, filtre plus le bruit)
- **0.75 - 1.0**: Mode "VeryAggressive" (très agressif, ne conserve que la voix claire)

**Défaut**: `0.5`

### `--max-chunk-ms <DURATION>`
Durée maximale d'un chunk audio en millisecondes. Après cette durée de silence, l'utterance est considérée comme terminée et sauvegardée.

**Défaut**: `1000` (1 seconde)

### `--output-dir <PATH>`
Répertoire où sauvegarder les fichiers WAV des utterances détectées.

**Défaut**: `/tmp/audio_chunks`

## Exemples

### Mode par défaut
```bash
RUST_LOG=info cargo run -- --stream
```

### Mode très sensible avec chunks longs
```bash
RUST_LOG=info cargo run -- --stream --vad-threshold 0.8 --max-chunk-ms 3000
```

### Mode peu sensible avec chunks courts
```bash
RUST_LOG=info cargo run -- --stream --vad-threshold 0.3 --max-chunk-ms 500
```

### Avec répertoire de sortie personnalisé
```bash
RUST_LOG=info cargo run -- --stream --output-dir ~/Documents/recordings
```

## Logging

Le niveau de log peut être contrôlé avec la variable d'environnement `RUST_LOG`:
- `RUST_LOG=error`: Erreurs seulement
- `RUST_LOG=warn`: Warnings et erreurs
- `RUST_LOG=info`: Informations générales (recommandé)
- `RUST_LOG=debug`: Détails supplémentaires

## Fonctionnement

1. Le système capture le flux audio du microphone par défaut
2. Le VAD analyse l'audio en frames de 30ms (480 samples à 16kHz)
3. Lorsque de la voix est détectée, l'enregistrement commence
4. L'enregistrement continue tant que la voix est présente
5. Après `max-chunk-ms` millisecondes de silence, l'utterance est sauvegardée
6. Les fichiers sont nommés `utterance_0001.wav`, `utterance_0002.wav`, etc.

## Format de sortie

- **Format**: WAV PCM
- **Sample rate**: 16kHz
- **Channels**: 1 (mono)
- **Bit depth**: 16-bit

## Arrêt

Appuyez sur `Ctrl+C` pour arrêter l'enregistrement.
