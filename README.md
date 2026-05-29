# Feelsay

Lightweight cross-platform desktop foundation for local-first real-time captions and translation.

## Current Scope

PBI-001 creates the app shell only:

- Tauri 2 desktop app with Rust backend and Svelte/TypeScript frontend
- Dashboard placeholder for source selection and settings
- Separate translucent caption overlay placeholder
- Rust-owned settings persisted locally in SQLite
- Stub modules for future audio capture, ASR, translation, overlay, transcript, and settings work

No audio capture, ASR, translation, diarization, export, mobile app, or cloud service is implemented yet.

## Requirements

- Node.js 22+
- npm 10+
- Rust/Cargo
- Tauri desktop prerequisites for your OS

Windows is the first target. macOS and Linux are kept in mind, but the overlay transparency behavior may need platform-specific follow-up.

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

## Local Data

Settings are stored by the Rust core in the app data directory as `feelsay.sqlite3`. The frontend does not write settings directly.

## Manual PBI-001 Test

1. Run `npm run tauri dev`.
2. Confirm the dashboard opens and looks like the Feelsay app, not the Tauri template.
3. Confirm `Start Captions` is disabled.
4. Click `Preview Overlay`.
5. Confirm the overlay opens as a separate translucent, resizable, always-on-top placeholder window.
