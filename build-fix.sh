#!/bin/bash

# Clean previous builds
cd src-tauri
cargo clean -p whisper-rs-sys
cd ..

# Set environment variables for macOS 11.0 target
export MACOSX_DEPLOYMENT_TARGET=11.0
export CXXFLAGS="-mmacosx-version-min=11.0"
export CFLAGS="-mmacosx-version-min=11.0"
export LDFLAGS="-mmacosx-version-min=11.0"

# Build
pnpm tauri build --bundles dmg
