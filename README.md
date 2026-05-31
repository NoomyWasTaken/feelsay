# Feelsay

Lightweight cross-platform desktop foundation for local-first real-time captions and translation.

## Current Scope

The app currently contains the shell only:

- Tauri 2 desktop app with Rust backend and Svelte/TypeScript frontend
- Minimal dashboard with source selection and Start/Stop controls
- Rust platform/source scaffolding
- Mock audio meter scaffolding
- Stub modules for future audio capture, ASR, translation, and transcript work

The Caption Overlay implementation has been intentionally purged and documented in `docs/caption-overlay.md` so it can be rebuilt from a clean baseline.

No audio capture, ASR, translation, diarization, export, mobile app, or cloud service is implemented yet.

## Requirements

- Node.js 22+
- npm 10+
- Rust/Cargo
- Tauri desktop prerequisites for your OS

Windows is the first target. macOS and Linux are kept in mind.

## Setup

```powershell
npm install
```

## Run

```powershell
npm run tauri dev
```

## Build

```powershell
npm run build
npm run tauri build
```

## Checks

```powershell
npm run format:check
npm run lint
npm run rust:lint
cargo test --manifest-path src-tauri/Cargo.toml
```

## Manual Test

1. Run `npm run tauri dev`.
2. Confirm the dashboard opens and looks like the Feelsay app, not the Tauri template.
3. Confirm `Start` is disabled until a source is selected.
4. Select a source.
5. Confirm `Start` and `Stop` update the shell state without creating a Caption Overlay window.
