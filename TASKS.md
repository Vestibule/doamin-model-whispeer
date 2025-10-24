# Tasks

## Completed
- [x] **Fix macOS Build Compilation Errors** (2025-01-24)
  - Removed `_gpu` feature from `whisper-rs` dependency in `Cargo.toml`
  - Configured `MACOSX_DEPLOYMENT_TARGET=11.0` in `src-tauri/.cargo/config.toml`
  - Resolved C++ filesystem availability errors for macOS 10.15
  - Successfully built DMG installer for macOS ARM64

- [x] **Implement Audio Level Normalization (AGC)** (2025-01-24)
  - Added Automatic Gain Control (AGC) to normalize audio input levels
  - Configured default gain multiplier of 3.0x for better microphone sensitivity
  - Implemented adaptive gain adjustment with peak tracking and smoothing
  - Target level set to 50% of maximum for optimal transcription quality
  - AGC enabled by default to handle low microphone input levels

## In Progress

## Backlog
