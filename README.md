# Feelsay

Lightweight Windows-first desktop foundation for local-first real-time captions and translation.

## Current Scope

The app currently includes:

- Tauri 2 desktop shell with Rust backend and Svelte/TypeScript frontend
- Minimal main window with source selection, Start/Stop, audio meter, and Settings
- Windows-first source/device enumeration scaffolding
- Real Windows microphone and system/output audio level preview
- VAD-based speech/silence status
- Child-process Caption Overlay window on Windows
- Overlay settings with live autosave, five profiles, visual placement, and click-through mode
- Model management metadata for local Whisper models
- Local ASR through an external `whisper.cpp` executable path
- Caption, translate-to-English, and original-plus-English caption modes
- Stub modules for future translation, transcripts, and platform expansion

No production diarization, mobile app, or cloud service is implemented yet.

## Docs

- Product summary: [`docs/SUMMARY.md`](docs/SUMMARY.md)
- Product/technical spec PDF: [`docs/realtime_caption_translation_app_spec.pdf`](docs/realtime_caption_translation_app_spec.pdf)
- Caption Overlay notes: [`docs/caption-overlay.md`](docs/caption-overlay.md)
- Product backlog: [`docs/BACKLOG.md`](docs/BACKLOG.md)
- Installer smoke test: [`docs/installer-smoke-test.md`](docs/installer-smoke-test.md)

## Requirements

- Node.js 22+
- npm 10+
- Rust/Cargo
- Tauri desktop prerequisites for your OS

Windows is the first target. macOS and Linux should stay possible through platform adapters.

Optional local ASR builds use `whisper-rs` behind the `local-asr` Cargo feature. On Windows, that feature requires LLVM/libclang and `LIBCLANG_PATH` pointing to the folder containing `libclang.dll`.

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

## Manual Smoke Test

1. Run `npm run tauri dev`.
2. Select a source.
3. Click `Start`.
4. Confirm the Caption Overlay opens.
5. Open Settings and adjust overlay appearance.
6. Confirm settings autosave and update the running overlay.
7. Click `Stop` and confirm the overlay closes.
