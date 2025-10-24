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

- [x] **Optimize Whisper Model Selection** (2025-01-24)
  - Downloaded ggml-small.bin (466MB) as default - good speed/quality balance
  - Downloaded ggml-medium.bin (1.5GB) as alternative for better quality
  - Replaced ggml-small-q5_1.bin (quantified, less accurate)
  - Default model: small (3x faster than medium, still accurate for French)
  - Can switch to medium via WHISPER_MODEL_PATH environment variable

## In Progress

## Backlog
