# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

This is a Tauri application with a Svelte 5 + TypeScript frontend and Rust backend. The UI uses Flowbite Svelte component library with Tailwind CSS. Tauri combines web frontend technologies with Rust to create lightweight, secure desktop applications.

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
- TypeScript type checking is integrated into the build process

### Rust Backend
Navigate to `src-tauri/` for Rust commands:
- `cargo build` - Build Rust backend
- `cargo test` - Run Rust tests
- `cargo clippy` - Lint Rust code
- `cargo fmt` - Format Rust code

## Architecture

### Frontend (Svelte 5 + TypeScript + Flowbite)
- **Entry point**: `src/main.js` - Mounts Svelte application
- **Root component**: `src/App.svelte` - Uses Svelte 5 runes ($state, $effect, etc.)
- **Vite config**: `vite.config.ts` - Configured for Tauri with fixed port 1420
- **UI Library**: Flowbite Svelte v0.46.23 with Tailwind CSS v3
- **Styling**: 
  - `src/app.css` - Tailwind directives
  - `tailwind.config.js` - Tailwind configuration with Flowbite plugin
  - `postcss.config.js` - PostCSS configuration
- **Icons**: flowbite-svelte-icons for professional icon set
- **Components**: All UI uses Flowbite components (Card, Button, Alert, Input, etc.)

### Backend (Rust + Tauri)
- **Entry point**: `src-tauri/src/main.rs` - Launches the application
- **Library**: `src-tauri/src/lib.rs` - Contains Tauri commands and app setup
- **Commands**: Rust functions marked with `#[tauri::command]` can be invoked from frontend using `invoke()` from `@tauri-apps/api/core`

### Frontend-Backend Communication
Tauri commands are the primary way the frontend communicates with the backend:
1. Define Rust function with `#[tauri::command]` in `src-tauri/src/lib.rs`
2. Register handler in `tauri::Builder` using `invoke_handler(tauri::generate_handler![command_name])`
3. Call from Svelte using `invoke("command_name", { args })` from `@tauri-apps/api/core`

### Configuration
- `src-tauri/tauri.conf.json` - Main Tauri configuration (app metadata, build commands, window settings)
- `package.json` - Frontend dependencies and npm scripts
- `src-tauri/Cargo.toml` - Rust dependencies and package metadata

## UI Development Guidelines

### Using Flowbite Svelte
- Always use Flowbite components instead of custom HTML/CSS
- Import components from `'flowbite-svelte'`
- Import icons from `'flowbite-svelte-icons'`
- Use Tailwind utility classes for spacing and layout
- Follow Flowbite's prop conventions (size, color, border, etc.)

### Common Flowbite Components
- **Card**: Container for content sections
- **Button**: Interactive buttons with size and color variants
- **Input/Textarea**: Form inputs with consistent styling
- **Alert**: Messages and notifications
- **Select**: Dropdown selections
- **Spinner**: Loading indicators
- **Heading**: Semantic headings (h1-h6)

### Styling Best Practices
- Use Tailwind utility classes for layout (flex, grid, spacing)
- Leverage Flowbite's dark mode support (automatic with dark: prefix)
- Custom colors defined in `tailwind.config.js` (primary, secondary)
- No custom CSS needed - everything via Tailwind + Flowbite

### Documentation
- See `REFACTOR_SUMMARY.md` for migration details
- See `UI_IMPROVEMENTS.md` for visual enhancements guide
- [Flowbite Svelte Docs](https://flowbite-svelte.com/)
- [Tailwind CSS Docs](https://tailwindcss.com/docs)

## Build Troubleshooting

### macOS Compilation Issues
If you encounter C++ filesystem availability errors during build (e.g., "'path' is unavailable: introduced in macOS 10.15"):

1. Ensure `whisper-rs` in `src-tauri/Cargo.toml` does NOT use the `_gpu` feature (known to cause macOS 10.15 compatibility issues)
2. Clean the build cache: `cd src-tauri && cargo clean`
3. Build with explicit deployment target: `MACOSX_DEPLOYMENT_TARGET=11.0 pnpm tauri build --bundles dmg`

The `src-tauri/.cargo/config.toml` is configured with `MACOSX_DEPLOYMENT_TARGET=11.0` to prevent these issues.

## Wrap Rules
To test the build, use `build`, not `dev`
