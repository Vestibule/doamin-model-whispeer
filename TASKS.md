# Tasks

## Completed
- [x] **Fix macOS Build Compilation Errors** (2025-01-24)
  - Removed `_gpu` feature from `whisper-rs` dependency in `Cargo.toml`
  - Configured `MACOSX_DEPLOYMENT_TARGET=11.0` in `src-tauri/.cargo/config.toml`
  - Resolved C++ filesystem availability errors for macOS 10.15
  - Successfully built DMG installer for macOS ARM64

- [x] **Implement Audio Level Normalization (AGC)** (2025-01-24)
  - Added Automatic Gain Control (AGC) to normalize audio input levels
  - Configured default gain multiplier of 2.0x for better microphone sensitivity
  - Implemented adaptive gain adjustment with peak tracking and smoothing
  - Target level set to 30% of maximum to avoid clipping and distortion
  - AGC enabled by default to handle low microphone input levels

- [x] **Implement True Push-to-Talk Mode** (2025-01-24)
  - Added push_to_talk configuration flag to AudioSessionConfig
  - In PTT mode: records entire audio stream from start to stop without VAD segmentation
  - Saves single utterance file when recording stops (spacebar release)
  - Tauri app uses PTT mode by default for better control and quality
  - CLI mode still uses VAD-based automatic segmentation

- [x] **Upgrade Whisper Model to Medium** (2025-01-24)
  - Downloaded ggml-medium.bin (1.5GB) for better transcription accuracy
  - Replaced ggml-small-q5_1.bin (quantified, less accurate) with full precision model
  - Medium model provides significantly better French transcription
  - Default model path updated in lib.rs

## In Progress

## Backlog
