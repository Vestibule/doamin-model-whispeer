# Tasks

## Completed
- [x] **Fix macOS Build Compilation Errors** (2025-01-24)
  - Removed `_gpu` feature from `whisper-rs` dependency in `Cargo.toml`
  - Configured `MACOSX_DEPLOYMENT_TARGET=11.0` in `src-tauri/.cargo/config.toml`
  - Resolved C++ filesystem availability errors for macOS 10.15
  - Successfully built DMG installer for macOS ARM64

## In Progress

## Backlog
