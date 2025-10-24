# Whisper Models

This directory contains Whisper speech-to-text models.

## Current Model

- **ggml-small.bin** (465MB) - Default model, good balance speed/quality for French
- ggml-medium.bin (1.5GB) - Available, better quality but slower
- ~~ggml-small-q5_1.bin~~ (181MB) - Deprecated, quantified and less accurate

## Available Models

Download from [HuggingFace](https://huggingface.co/ggerganov/whisper.cpp/tree/main):

| Model      | Size   | Quality           | Speed    | Best For       |
|------------|--------|-------------------|----------|----------------|
| tiny       | 75 MB  | ★☆☆☆☆            | ★★★★★    | Testing only   |
| base       | 142 MB | ★★☆☆☆            | ★★★★☆    | English only   |
| **small**  | **466 MB** | **★★★☆☆**     | **★★★☆☆**| **French** ✓   |
| medium     | 1.5 GB | ★★★★☆            | ★★☆☆☆    | Better quality |
| large-v3   | 2.9 GB | ★★★★★            | ★☆☆☆☆    | Best quality   |

## Download Commands

```bash
# Small (recommended - good balance speed/quality for French)
curl -L -o models/whisper/ggml-small.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin

# Medium (better quality but slower)
curl -L -o models/whisper/ggml-medium.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin

# Large-v3 (best quality)
curl -L -o models/whisper/ggml-large-v3.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin
```

## Configuration

Set via environment variable:
```bash
export WHISPER_MODEL_PATH="models/whisper/ggml-medium.bin"
```

Or edit `src-tauri/src/lib.rs` line 158.
