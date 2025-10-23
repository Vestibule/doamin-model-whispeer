# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

This is a Tauri application with a Vue 3 + TypeScript frontend and Rust backend. Tauri combines web frontend technologies with Rust to create lightweight, secure desktop applications.

## Package Manager

This project uses **pnpm** for JavaScript dependencies. Do not use npm or yarn.

## Commands

### Development
- `pnpm dev` - Start Vite dev server (frontend only)
- `pnpm tauri dev` - Run full Tauri app with hot reload (builds Rust backend + runs frontend)

### Building
- `pnpm build` - Build frontend (TypeScript type checking + Vite build)
- `pnpm tauri build` - Build production Tauri application with bundled installers

### Type Checking
- `pnpm vue-tsc --noEmit` - Run TypeScript compiler to check for type errors without emitting files

### Rust Backend
Navigate to `src-tauri/` for Rust commands:
- `cargo build` - Build Rust backend
- `cargo test` - Run Rust tests
- `cargo clippy` - Lint Rust code
- `cargo fmt` - Format Rust code

## Architecture

### Frontend (Vue 3 + TypeScript)
- **Entry point**: `src/main.ts` - Bootstraps Vue application
- **Root component**: `src/App.vue` - Uses `<script setup>` composition API
- **Vite config**: `vite.config.ts` - Configured for Tauri with fixed port 1420
- **TypeScript**: Strict mode enabled with comprehensive linting rules

### Backend (Rust + Tauri)
- **Entry point**: `src-tauri/src/main.rs` - Launches the application
- **Library**: `src-tauri/src/lib.rs` - Contains Tauri commands and app setup
- **Commands**: Rust functions marked with `#[tauri::command]` can be invoked from frontend using `invoke()` from `@tauri-apps/api/core`

### Frontend-Backend Communication
Tauri commands are the primary way the frontend communicates with the backend:
1. Define Rust function with `#[tauri::command]` in `src-tauri/src/lib.rs`
2. Register handler in `tauri::Builder` using `invoke_handler(tauri::generate_handler![command_name])`
3. Call from Vue using `invoke("command_name", { args })`

### Configuration
- `src-tauri/tauri.conf.json` - Main Tauri configuration (app metadata, build commands, window settings)
- `package.json` - Frontend dependencies and npm scripts
- `src-tauri/Cargo.toml` - Rust dependencies and package metadata
