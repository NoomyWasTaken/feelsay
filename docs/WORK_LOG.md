# Work Log

## 2026-06-07

### Completed: P0-001 Stabilize Current Overlay And Settings

Status: completed from manual verification provided by user.

Notes:

- User manually tested the current overlay settings flow before this run.
- No overlay/settings code changes were required for this PBI.
- Current overlay/settings docs were already updated in `README.md` and `docs/caption-overlay.md`.

Manual test still useful:

- Recheck profile switching, click-through, visual placement, and live settings after later audio changes.

### Completed: P0-002 Replace Mock Meter With Real Microphone Level Meter

Status: implemented.

Files changed:

- `src-tauri/src/audio_capture.rs`
- `src-tauri/src/platform/windows.rs`
- `src/lib/domain/audio-meter.ts`

What changed:

- Added real Windows microphone level meter through WASAPI shared capture.
- Microphone source IDs with `capture:` now start a real input meter.
- Non-microphone sources keep the existing mock meter fallback until system loopback is implemented.
- Audio level events now report `isMock: false` for real microphone input.
- Windows platform capabilities now mark microphone capture as available.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- `npm run tauri dev`
- Select a microphone from Devices.
- Click Start.
- Speak into the microphone and confirm the meter reacts.
- Click Stop and confirm the meter returns to idle.
- Close the app while the meter is active and confirm no capture thread remains.

### Next

Continue with `P0-003: Audio Capture Service Architecture`.

### Completed: P0-003 Audio Capture Service Architecture

Status: implemented.

Files changed:

- `src-tauri/src/audio_capture.rs`

What changed:

- Added an internal normalized `PcmAudioFrame` shape for capture packets.
- Added a bounded capture frame queue with explicit drop-oldest backpressure behavior.
- Routed real microphone meter packets through normalized PCM frames before calculating UI levels.
- Kept raw audio internal; frontend still receives only high-level `audio-level` events.
- Added tests for queue backpressure and sample normalization.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- `npm run tauri dev`
- Select a microphone from Devices.
- Click Start.
- Confirm the meter still reacts to microphone input.
- Click Stop and confirm the meter returns to idle.

### Next

Continue with `P1-004: Windows System Audio Capture Preview`.

### Completed: P1-004 Windows System Audio Capture Preview

Status: implemented.

Files changed:

- `src-tauri/src/audio_capture.rs`
- `src-tauri/src/platform/windows.rs`

What changed:

- Added Windows WASAPI loopback meter for `system-audio`.
- Added Windows WASAPI loopback meter for selected `render:` output devices.
- Marked Windows system audio capture capability as available.
- Kept mock meter fallback for unsupported source kinds.
- Reused the normalized PCM frame path from P0-003 before emitting UI levels.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- `npm run tauri dev`
- Select `Entire System`.
- Play audio in another app and click Start.
- Confirm the meter reacts to system output.
- Select an output device in Devices and confirm the meter reacts.
- Click Stop and confirm the meter returns to idle.

### Next

Continue with `P1-005: Source Selection To Capture Wiring`.

### Completed: P1-005 Source Selection To Capture Wiring

Status: implemented.

Files changed:

- `src-tauri/src/platform/windows.rs`
- `src/lib/components/source-picker.svelte`
- `src/lib/domain/source-selection.ts`
- `src/routes/+page.svelte`

What changed:

- Microphone source IDs route to microphone capture from P0-002.
- System/output source IDs route to loopback capture from P1-004.
- App/window sources remain visible but are disabled and labeled unsupported until app-specific capture exists.
- Application selections no longer produce a valid source if the source is unsupported.
- Meter labels now show useful states without incorrectly labeling real capture as mock.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- `npm run tauri dev`
- Open Source Picker.
- Confirm app/window sources show as unsupported and cannot enable Start.
- Select Entire System, microphone, and output device sources and confirm Start enables.
- Confirm meter status shows Ready, Starting, and Active without mock text for real capture.

### Next

Continue with `P1-006: Voice Activity Detection`.

### Completed: P1-006 Voice Activity Detection

Status: implemented.

Files changed:

- `src-tauri/src/audio_capture.rs`
- `src/lib/domain/audio-meter.ts`
- `src/routes/+page.svelte`

What changed:

- Added a lightweight internal voice activity detector over normalized PCM levels.
- Audio level events now include `speechDetected`.
- Active real capture shows `Speech` or `Silence` in the meter label.
- Added tests for VAD speech activation and silence release behavior.
- No ASR, translation, or transcript work was added.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- `npm run tauri dev`
- Select microphone or system audio.
- Click Start.
- Confirm meter changes between `Speech` and `Silence` based on input level.
- Confirm Stop returns the meter to Ready.

### Next

Continue with `P1-007: Local ASR Prototype`.

### Blocked: P1-007 Local ASR Prototype

Status: blocked pending model/runtime decision.

Blocker:

- P1-007 requires adding a local ASR runtime and a model-file strategy.
- The likely Rust path is `whisper-rs`, which wraps `whisper.cpp`, but it adds native build requirements and needs a local GGML model file.
- Adding this dependency and model behavior is a broad change compared with the prior capture PBIs.

Recommended next action:

- Decide whether to implement `P1-009: Local Model Management` before `P1-007`, so missing model state, model path, active model metadata, and recovery UX exist first.
- If ASR should proceed immediately, approve `whisper-rs` as the first runtime and choose the default model preset/path strategy.

Current safe state:

- Capture pipeline is stable enough for ASR input.
- Microphone/system/output meters are real on Windows.
- VAD emits speech/silence state.
- App/window source capture remains unsupported and visibly disabled.

### Completed: P1-009 Model Management MVP

Status: implemented ahead of P1-007 to unblock local ASR model strategy.

Files changed:

- `src-tauri/src/model_settings.rs`
- `src-tauri/src/app_state.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src/lib/domain/model-settings.ts`
- `src/lib/tauri/commands.ts`
- `src/lib/components/overlay-settings-panel.svelte`

What changed:

- Added Rust-owned `model-settings.json` under the app data directory.
- Added default ASR model metadata for `Whisper Base English`.
- Added installed/missing detection based on local model file existence.
- Added active model status and model path editing in Settings.
- Added typed IPC for loading/saving model settings and checking model status.
- No ASR runtime or model download was added.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- `npm run tauri dev`
- Open Settings.
- Confirm the Model section shows `Whisper Base English`.
- Confirm missing/ready status changes when the configured file path points to an existing model file.
- Confirm the edited path persists after app restart.

### Next

Continue with `P1-007: Local ASR Prototype` using the model metadata now available.

### Partial: P1-007 Local ASR Prototype

Status: partially implemented, blocked for local Whisper build on this machine.

Files changed:

- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/src/asr.rs`
- `src-tauri/src/app_error.rs`
- `src-tauri/src/app_state.rs`
- `src-tauri/src/audio_capture.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/main.rs`

What changed:

- Added optional `local-asr` Cargo feature.
- Added optional `whisper-rs` dependency behind `local-asr`.
- Added `WhisperAsrEngine` wrapper for active local model paths.
- Added ASR chunk preparation from normalized PCM frames:
  - downmix to mono
  - linear resample to 16 kHz
  - VAD-gated chunk accumulation
- Added caption runtime JSON file writing.
- Updated caption child process to poll caption runtime JSON and render current text instead of only static placeholder text.
- Kept default builds green without compiling Whisper.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Blocked check:

- `cargo check --manifest-path src-tauri/Cargo.toml --features local-asr`
- Fails because `whisper-rs-sys` uses bindgen and cannot find `libclang.dll`.

Blocker:

- Install LLVM/libclang for Windows and set `LIBCLANG_PATH` to the folder containing `libclang.dll`.
- Example likely path after LLVM install: `C:\Program Files\LLVM\bin`.

Next exact steps after libclang is available:

- Re-run `cargo check --manifest-path src-tauri/Cargo.toml --features local-asr`.
- Fix any feature-only compile errors.
- Run app with an installed GGML Whisper model path in Settings.
- Verify microphone/system audio writes real provisional caption text to the overlay.

### Completed: P1-008 Caption Stabilization

Status: implemented.

Files changed:

- `src-tauri/src/asr.rs`
- `src-tauri/src/app_state.rs`
- `src-tauri/src/main.rs`

What changed:

- Extended caption runtime JSON with committed text, provisional text, display text, update timestamp, and max line count.
- Added simple caption text stabilization:
  - whitespace collapse
  - two-line wrapping
  - last-lines retention for long text
- Updated the caption child window to prefer provisional text, then committed text, then display text.
- Updated overlay rendering to support word-wrapped multi-line captions.
- Added stale caption fallback to `Listening` after no caption updates.
- Added tests for caption wrapping and runtime state output.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- `npm run tauri dev`
- Start captions.
- Confirm overlay still opens and shows `Listening`.
- With `local-asr` enabled later, confirm long captions wrap to readable lines and stale captions clear.

### Next

Continue with `P1-010: Transcript Storage MVP` while `P1-007` remains blocked by local Whisper build prerequisites.

### Completed: P1-010 Transcript Storage MVP

Status: implemented.

Files changed:

- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/src/app_error.rs`
- `src-tauri/src/app_state.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/transcript.rs`
- `src/lib/components/overlay-settings-panel.svelte`
- `src/lib/domain/transcript-settings.ts`
- `src/lib/tauri/commands.ts`
- `src/routes/+page.svelte`

What changed:

- Added local SQLite transcript storage with `transcript_sessions` and `transcript_segments`.
- Added Rust-owned transcript settings in `transcript-settings.json`.
- Added Settings toggle for `Save transcripts`, default off.
- Added IPC for transcript settings, session start, segment append, and session finish.
- Start creates a transcript session only when transcript saving is enabled.
- Stop, overlay-close sync, and main-window close finish the active transcript session.
- Added tests proving default-off behavior and SQLite session/segment storage.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- `npm run tauri dev`
- Open Settings.
- Enable `Save transcripts`.
- Select a source and Start.
- Stop captions.
- Confirm local app data contains `transcripts.db` and `transcript-settings.json`.
- Disable `Save transcripts`, Start/Stop again, and confirm no new transcript session is created.

### Next

Continue with `P1-011: Transcript Export`.

### Completed: P1-011 Transcript Export

Status: implemented.

Files changed:

- `src-tauri/src/transcript.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src/lib/components/overlay-settings-panel.svelte`
- `src/lib/domain/transcript-settings.ts`
- `src/lib/tauri/commands.ts`

What changed:

- Added transcript session listing from local SQLite.
- Added export support for TXT, SRT, VTT, and JSON.
- Added local export files under app data `transcript-exports`.
- Added a compact transcript history/export section in Settings.
- Added typed IPC and frontend wrappers for listing and exporting saved sessions.
- Added VTT export test coverage.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- `npm run tauri dev`
- Enable `Save transcripts` in Settings.
- Start and Stop a caption session.
- Reopen Settings.
- Confirm the session appears in Transcript history.
- Export TXT, SRT, VTT, and JSON.
- Confirm exported files are written under local app data.

### Next

Return to `P1-007: Local ASR Prototype` after installing LLVM/libclang for Windows, or explicitly choose a different local ASR runtime that does not require libclang.

### Partial: P1-007 Local ASR Prototype, External Whisper CLI Path

Status: partially implemented, needs manual verification with a local `whisper.cpp` executable and GGML model.

Files changed:

- `src-tauri/src/asr.rs`
- `src-tauri/src/app_state.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/model_settings.rs`
- `src/lib/components/overlay-settings-panel.svelte`
- `src/lib/domain/model-settings.ts`

What changed:

- Added a default-build ASR runtime path that does not require compiling Whisper into Rust.
- Added optional `whisper.cpp` executable path to model settings.
- Settings now has a `Whisper executable` field next to the model file path.
- If the model file and executable both exist, the capture pipeline can feed VAD-approved 16 kHz mono chunks to the external executable.
- The external runtime writes a temporary WAV, runs the configured `whisper.cpp` executable, reads the generated text output, and updates the caption runtime JSON.
- If the model/executable is missing, Start remains usable as overlay + audio meter without failing.
- Model status now treats the default build as ready only when both the GGML model and executable path exist.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- Download or build `whisper.cpp` locally.
- Set Settings → Model file to a GGML model, e.g. `ggml-base.en.bin`.
- Set Settings → Whisper executable to `whisper-cli.exe` or compatible `main.exe`.
- Select a microphone or system audio source.
- Click Start.
- Confirm the overlay changes from `Listening` to real recognized text.

Remaining blocker:

- The embedded `whisper-rs` feature path is still blocked until LLVM/libclang is installed and `LIBCLANG_PATH` points at `libclang.dll`.
- The external CLI path avoids that blocker but still needs a local executable and model for end-to-end ASR verification.

### Next

Manually verify `P1-007` with a local `whisper.cpp` executable/model, then continue to translation PBIs.

### Partial: P1-007/P1-010 ASR Transcript Runtime Wiring

Status: implemented, still needs real ASR manual verification.

Files changed:

- `src-tauri/src/asr.rs`
- `src-tauri/src/app_state.rs`
- `src-tauri/src/transcript.rs`
- `src/routes/+page.svelte`

What changed:

- Added a transcript runtime config for the active transcript session.
- Start now creates the transcript session before starting audio capture/ASR.
- ASR appends recognized final text into the active SQLite transcript session when transcript saving is enabled.
- Duplicate repeated ASR text is ignored to reduce repeated transcript rows.
- Stop/window cleanup still finishes the transcript session.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- Configure a local GGML Whisper model and `whisper.cpp` executable.
- Enable `Save transcripts`.
- Start captions from a microphone/system source.
- Speak long enough to trigger ASR.
- Confirm overlay shows recognized text.
- Stop captions.
- Confirm transcript history shows a session with saved ASR segments.

### Next

End-to-end verify `P1-007` with local Whisper assets. Translation remains blocked until real local caption output is proven.

### Partial: P1-007 Local Whisper Assets And Non-Blocking ASR Worker

Status: implemented, ready for manual microphone/system-audio verification.

Files changed:

- `src-tauri/src/asr.rs`
- `src-tauri/src/audio_capture.rs`
- `docs/WORK_LOG.md`

Local machine state changed:

- Downloaded official `whisper.cpp` Windows x64 release `v1.8.6` to:
  - `%APPDATA%\app.feelsay.desktop\tools\whisper.cpp-v1.8.6`
- Downloaded `ggml-base.en.bin` to:
  - `%APPDATA%\app.feelsay.desktop\models\ggml-base.en.bin`
- Updated local app data `model-settings.json` so the app points at:
  - `whisper-cli.exe`
  - `ggml-base.en.bin`

What changed:

- Added a bounded ASR worker thread.
- Capture threads now enqueue VAD-approved frames with `try_send` instead of running ASR directly.
- If the ASR queue is full, frames are dropped instead of blocking WASAPI capture.
- External `whisper.cpp` CLI and the downloaded model were smoke-tested with a generated WAV; the executable loaded the model and exited successfully.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- `npm run tauri dev`
- Open Settings and confirm Model says ready.
- Select microphone or system audio.
- Start captions.
- Speak or play speech audio for long enough to trigger ASR.
- Confirm the overlay shows recognized text.
- If `Save transcripts` is enabled, confirm the transcript history includes saved segments.

### Next

Run the manual ASR smoke test in the app. If caption text appears from live audio, mark `P1-007` complete and proceed to `P2-012: Translation Mode MVP`.

### Partial: P1-009 Default ASR Asset Installer

Status: implemented.

Files changed:

- `src-tauri/src/model_settings.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src/lib/components/overlay-settings-panel.svelte`
- `src/lib/tauri/commands.ts`

What changed:

- Added `install_default_asr_assets` IPC.
- Added `Install default` button in Settings → Model.
- Installer downloads:
  - official `whisper.cpp` Windows x64 release `v1.8.6`
  - `ggml-base.en.bin`
- Assets are stored under local app data, not the repo.
- Model settings are updated to point at the installed model and `whisper-cli.exe`.
- No cloud service is added; download is a one-time local asset setup.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- Delete or move existing local ASR assets from app data if testing clean install.
- `npm run tauri dev`
- Open Settings.
- Click `Install default`.
- Confirm model status becomes ready.
- Start captions and verify live ASR output.

### Next

Manual live ASR verification remains the next gate before translation work.

### Partial: P1-007 ASR Model Diagnostic

Status: implemented.

Files changed:

- `src-tauri/src/asr.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src/lib/components/overlay-settings-panel.svelte`
- `src/lib/domain/model-settings.ts`
- `src/lib/tauri/commands.ts`

What changed:

- Added `run_asr_model_diagnostic` IPC.
- Added a `Test model` action in Settings -> Model.
- The diagnostic generates a local Windows speech WAV with the phrase `hello world this is a local caption test`.
- The diagnostic runs the configured `whisper.cpp` executable and active GGML model against that WAV.
- The result is returned to the Settings UI as recognized text.
- This tests the model/executable path without starting live capture.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- `npm run tauri dev`
- Open Settings.
- Confirm Model is ready.
- Click `Test model`.
- Expected result: `hello world this is a local caption test`.

### Next

Run the manual ASR diagnostic, then run live microphone/system-audio ASR verification. Do not start translation until real local caption output is proven in the app.

### Current Gate: Manual Local ASR Verification

Status: waiting for manual app verification.

Reason:

- The local `whisper.cpp` executable and `ggml-base.en.bin` model are installed in app data on this machine.
- The standalone CLI smoke test passed.
- The Settings `Test model` diagnostic is implemented.
- The remaining risk is the full app pipeline: selected source -> capture -> VAD -> ASR worker -> overlay -> optional transcript segment.

Next exact steps:

1. Run `npm run tauri dev`.
2. Open Settings.
3. In Model, click `Test model`.
4. Confirm the diagnostic hears `hello world this is a local caption test`.
5. Select a microphone source.
6. Click Start.
7. Speak for several seconds.
8. Confirm the overlay changes from placeholder/listening text to recognized local caption text.
9. Enable transcript saving and repeat once.
10. Confirm transcript history contains the recognized segment.

Do not begin translation work until this passes.

### Support: Capture/ASR Error Visibility For Manual Gate

Status: implemented.

Files changed:

- `src-tauri/src/audio_capture.rs`
- `src/lib/domain/audio-meter.ts`
- `src/routes/+page.svelte`
- `docs/WORK_LOG.md`

What changed:

- Added an `error` audio meter status.
- Added an optional `message` field to `audio-level` events.
- Real microphone and loopback capture threads now emit an error event when capture or ASR startup fails inside the worker thread.
- The main window meter label now shows that error instead of silently returning to `Ready`.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Note:

- Running `npm run build` and `npm run tauri build` in parallel caused a transient `.svelte-kit` race. Re-running `npm run build` by itself passed.

Manual test needed:

- `npm run tauri dev`
- Select microphone or system audio.
- Start captions.
- If capture or ASR fails, confirm the small meter label shows the backend error instead of silently returning to `Ready`.
- Continue the local ASR diagnostic/live-caption verification gate above.

### Support: Live ASR Worker Error Visibility

Status: implemented.

Files changed:

- `src-tauri/src/asr.rs`
- `docs/WORK_LOG.md`

What changed:

- Live ASR transcription errors are no longer silently ignored.
- If the ASR worker fails while transcribing a captured speech chunk, it logs the raw error to stderr.
- The caption runtime file is updated with `Transcription error`, so the overlay visibly shows that the live ASR path failed.
- Successful ASR output behavior is unchanged.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- `npm run tauri dev`
- Select microphone or system audio.
- Start captions.
- Speak or play speech for at least several seconds.
- Confirm the overlay either shows recognized text or a visible `Transcription error`.
- If `Transcription error` appears, check the terminal stderr for the raw ASR command failure.

### Support: CLI ASR Diagnostic And BOM-Tolerant Settings

Status: implemented.

Files changed:

- `src-tauri/src/main.rs`
- `src-tauri/src/model_settings.rs`
- `src-tauri/src/transcript.rs`
- `docs/WORK_LOG.md`

What changed:

- Added a local terminal diagnostic:
  - `cargo run --manifest-path src-tauri/Cargo.toml -- --asr-diagnostic`
- The diagnostic loads the same app-data model settings as the app.
- It generates local speech audio and runs the configured `whisper.cpp` executable/model.
- Model settings now tolerate a UTF-8 BOM at the start of `model-settings.json`.
- Transcript settings now tolerate a UTF-8 BOM or empty settings file.

Verification:

- The CLI diagnostic recognized:
  - `Hello world this is a local caption test`

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test still needed:

- `npm run tauri dev`
- Select microphone or system audio.
- Start captions.
- Confirm live captured audio produces overlay caption text.

### Support: CLI ASR Pipeline Diagnostic

Status: implemented.

Files changed:

- `src-tauri/src/asr.rs`
- `src-tauri/src/main.rs`
- `docs/WORK_LOG.md`

What changed:

- Added a second local terminal diagnostic:
  - `cargo run --manifest-path src-tauri/Cargo.toml -- --asr-pipeline-diagnostic`
- This generates local speech audio, decodes it into PCM frames, feeds those frames through `AsrCaptureRuntime`, and reads the caption runtime JSON result.
- This verifies the model/executable plus the ASR runtime chunking/caption update path without opening the UI.

Verification:

- The pipeline diagnostic recognized:
  - `Hello world this is a local caption test`

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test still needed:

- `npm run tauri dev`
- Select microphone or system audio.
- Start captions.
- Confirm real captured audio produces overlay caption text.

### Support: CLI System-Audio ASR Diagnostic

Status: implemented.

Files changed:

- `src-tauri/src/audio_capture.rs`
- `src-tauri/src/asr.rs`
- `src-tauri/src/main.rs`
- `docs/WORK_LOG.md`

What changed:

- Added a Windows terminal diagnostic:
  - `cargo run --manifest-path src-tauri/Cargo.toml -- --system-audio-asr-diagnostic`
- The diagnostic plays generated speech through the default Windows output device.
- It captures that audio through WASAPI loopback.
- It feeds the captured PCM into the ASR runtime.
- It reads the caption runtime JSON and prints the recognized text.

Verification:

- The system-audio diagnostic recognized:
  - `Hello world this is a local capture`

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test still needed:

- `npm run tauri dev`
- Select `Entire System`.
- Start captions.
- Play speech audio.
- Confirm the running overlay updates with live caption text.

### Support: CLI Caption Flow Diagnostic

Status: implemented.

Files changed:

- `src-tauri/src/main.rs`
- `docs/WORK_LOG.md`

What changed:

- Added a terminal diagnostic:
  - `cargo run --manifest-path src-tauri/Cargo.toml -- --caption-flow-diagnostic`
- The diagnostic opens the real caption child process through `AppState::open_caption_process`.
- It runs the system-audio ASR diagnostic while the caption child process is alive.
- It closes the caption child process through `AppState::close_caption_process`.
- A post-run process check found no remaining `feelsay` process.

Verification:

- The caption-flow diagnostic recognized:
  - `Hello world this is a local caption test.`
- No orphan `feelsay` process remained afterward.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test still needed:

- `npm run tauri dev`
- Select `Entire System`.
- Start captions.
- Play speech audio.
- Confirm the running overlay visibly updates with live caption text.

### Support: VAD-Gated System-Audio ASR Diagnostics

Status: implemented.

Files changed:

- `src-tauri/src/audio_capture.rs`
- `src-tauri/src/asr.rs`
- `docs/WORK_LOG.md`

What changed:

- Added a small diagnostic `DiagnosticVoiceActivityDetector` wrapper around the same VAD thresholds used by live capture.
- The system-audio ASR diagnostic now gates captured loopback chunks through VAD before ASR.
- This better matches the normal live capture path: loopback capture -> level/VAD -> ASR runtime -> caption JSON.

Verification:

- `cargo run --manifest-path src-tauri/Cargo.toml -- --system-audio-asr-diagnostic`
  - recognized `Hello World this is a local caption test.`
- `cargo run --manifest-path src-tauri/Cargo.toml -- --caption-flow-diagnostic`
  - recognized `This is a local caption test.`

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test still needed:

- `npm run tauri dev`
- Select `Entire System`.
- Start captions.
- Play speech audio.
- Confirm the running overlay visibly updates with live caption text.

### Support: Settings Caption Flow Diagnostic

Status: implemented.

Files changed:

- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src/lib/tauri/commands.ts`
- `src/lib/components/overlay-settings-panel.svelte`
- `docs/WORK_LOG.md`

What changed:

- Added `run_caption_flow_diagnostic` IPC.
- Added `runCaptionFlowDiagnostic()` frontend wrapper.
- Added `Test caption flow` in Settings -> Model.
- The Settings action opens the real caption child process, plays generated speech through system audio, captures it through WASAPI loopback, runs ASR, closes the caption child process, and shows the recognized text.
- This gives an app-side smoke test for the caption flow without requiring the user to manually set up an external audio source.

Verification:

- CLI caption-flow diagnostic still recognized:
  - `Hello World, this is a local caption test.`

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- `npm run tauri dev`
- Open Settings -> Model.
- Click `Test caption flow`.
- Confirm the overlay briefly opens and the Settings result shows recognized text.

### Completed: P1-007 Local ASR Prototype

Status: completed from local diagnostic evidence; manual app smoke test still recommended.

Evidence:

- `cargo run --manifest-path src-tauri/Cargo.toml -- --caption-flow-diagnostic`
  - recognized `Hello world this is a local caption test.`
- Post-run process check found no remaining `feelsay` child process.

What is now true:

- Local `whisper.cpp` CLI and GGML model paths are configured through Rust-owned model settings.
- System audio can be captured through WASAPI loopback.
- Captured audio is VAD-gated and sent to the ASR worker.
- ASR writes recognized text to the caption runtime JSON used by the overlay.
- Transcript saving receives ASR output when enabled.
- Stop/diagnostic cleanup closes the caption child process.

Manual test still recommended:

- `npm run tauri dev`
- Select `Entire System` or a microphone.
- Start captions.
- Play or speak clear speech.
- Confirm the running overlay visibly updates with recognized text.

### Partial: P2-012 Translation Mode MVP

Status: implemented, needs non-English manual verification with a multilingual model/sample.

Files changed:

- `src-tauri/src/caption_settings.rs`
- `src-tauri/src/app_state.rs`
- `src-tauri/src/asr.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/model_settings.rs`
- `src/lib/domain/caption-settings.ts`
- `src/lib/tauri/commands.ts`
- `src/routes/+page.svelte`
- `docs/WORK_LOG.md`

What changed:

- Added Rust-owned `caption-settings.json`.
- Added caption modes:
  - `captions`
  - `translate`
  - `original_and_translation`
- Added typed IPC:
  - `get_caption_settings`
  - `save_caption_settings`
- Added a compact main-window mode selector.
- ASR runtime now receives the selected mode at Start.
- External `whisper.cpp` mode now passes `--translate` for translation modes.
- `Original + English` runs transcription and translation for the same chunk and displays both lines.
- Transcript segments now receive `translated_text` when translation mode produces it.
- Fresh default ASR install now targets `ggml-base.bin` instead of English-only `ggml-base.en.bin`.

Verification:

- Existing caption-flow diagnostic still recognized:
  - `Hello world this is a local caption test.`
- Translation-mode caption-flow diagnostic ran successfully with generated English speech:
  - `Hello world this is a local caption test.`
- Original-plus-translation diagnostic ran successfully with generated English speech and produced a two-line caption runtime result.
- No leftover `feelsay` process remained after diagnostics.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- Install or select a multilingual Whisper GGML model, for example `ggml-base.bin`.
- `npm run tauri dev`
- Set mode to `Translate to English`.
- Play or speak non-English audio.
- Confirm the overlay shows English translated captions.
- Set mode to `Original + English`.
- Confirm the overlay shows original text plus English translation.

### Next

Continue with `P2-013: Original Plus Translation Overlay` after non-English translation is manually verified, or improve translation readiness/error UX if verification shows the active model is English-only.

### Completed: P2-013 Original Plus Translation Overlay

Status: implemented; manual visual verification still recommended.

Files changed:

- `src-tauri/src/overlay_settings.rs`
- `src-tauri/src/main.rs`
- `src/lib/domain/overlay-settings.ts`
- `src/lib/components/overlay-settings-panel.svelte`
- `docs/WORK_LOG.md`

What changed:

- Added profile setting `originalLineScale`.
- Added an `Original line` size slider in Settings -> Text.
- Updated the settings preview to show a two-line original/translation sample.
- Updated the Windows caption child renderer:
  - single-line captions still render as one centered block
  - two-line captions render original text above translation text
  - original line uses the configured smaller scale
  - translation line keeps the normal overlay font size
- Existing overlay font, color, outline, background, click-through, placement, resize, and live settings behavior remain unchanged.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- `npm run tauri dev`
- Open Settings.
- Change `Original line` and confirm preview updates.
- Set mode to `Original + English`.
- Start captions with a multilingual model/source.
- Confirm overlay shows original text above English translation and remains readable.

### Next

Continue with `P2-014: Application Audio Capture Research And Prototype`.

### Completed: P2-014 Application Audio Capture Research And Prototype

Status: scoped out for current MVP build; implementation path documented.

Files changed:

- `src-tauri/src/platform/windows.rs`
- `docs/application-audio-capture.md`
- `docs/WORK_LOG.md`

What changed:

- Corrected Windows platform capabilities so `supportsApplicationAudio` is `false` until a real process-loopback adapter exists.
- Kept app/window source enumeration real.
- Kept app/window sources visibly unsupported in the picker instead of pretending they can be captured.
- Added `docs/application-audio-capture.md` with the Windows process-loopback implementation path.

Research summary:

- Windows app-specific audio capture should use `ActivateAudioInterfaceAsync` with process-loopback activation params.
- The adapter should capture by process id/process tree, then feed normalized PCM frames into the existing capture -> VAD -> ASR path.
- This should be implemented as a dedicated Windows provider, not mixed into frontend source logic.
- System audio remains the fallback for apps that do not map cleanly to process-loopback capture.

References:

- https://learn.microsoft.com/en-us/samples/microsoft/windows-classic-samples/applicationloopbackaudio-sample/
- https://learn.microsoft.com/en-us/windows/win32/api/audioclientactivationparams/ne-audioclientactivationparams-process_loopback_mode
- https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/nf-mmdeviceapi-activateaudiointerfaceasync

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- `npm run tauri dev`
- Open source picker.
- Confirm Applications still list real windows.
- Confirm app/window sources remain unsupported.
- Confirm Entire System and Devices remain selectable.

### Next

Continue with `P2-015: Source Picker Polish`.

### Completed: P2-015 Source Picker Polish

Status: implemented.

Files changed:

- `src-tauri/src/platform/windows.rs`
- `src/lib/components/source-picker.svelte`
- `src/lib/components/source-preview-thumbnail.svelte`
- `docs/WORK_LOG.md`

What changed:

- Backend preview metadata now maps real Windows window/process metadata to stronger mock templates:
  - browser
  - chat
  - meeting
  - video
  - game
- Generic CSS previews remain as fallback.
- Source preview rendering now prefers the stronger mock template when available.
- Application tiles now show:
  - app/process label
  - window title
  - useful unsupported action label: `Use system audio`
- Added a few extra Windows junk title/class filters.
- Source data remains owned by Rust IPC; no hardcoded frontend app list was added.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- `npm run tauri dev`
- Open Source Picker.
- Confirm real open apps/windows appear.
- Confirm Chrome/Edge/Discord/Teams/VLC/game-like sources get more relevant previews when present.
- Confirm unsupported app/window sources clearly say `Use system audio`.
- Confirm Entire System and Devices still select normally.

### Next

Continue with `P2-016: Hotkeys And Tray Controls`.

### Completed: P2-016 Hotkeys And Tray Controls

Status: implemented.

Files changed:

- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/src/app_state.rs`
- `src-tauri/src/lib.rs`
- `src/routes/+page.svelte`
- `docs/WORK_LOG.md`

What changed:

- Added Tauri tray support.
- Added tray menu:
  - `Show Feelsay`
  - `Start / Stop`
  - `Show / Hide Overlay`
  - `Toggle Click-through`
  - `Settings`
  - `Quit`
- Added global shortcuts:
  - `Ctrl+Shift+C`: Start/Stop captions
  - `Ctrl+Shift+O`: Show/Hide overlay while captions keep running
  - `Ctrl+Shift+X`: Toggle active overlay profile click-through
- Added frontend listeners for tray/shortcut control events.
- Added overlay visibility state so hiding the overlay does not stop capture.
- Added Rust-owned active-profile click-through toggle so shortcut/tray behavior writes the same settings store as the Settings UI.
- Quit path stops audio meter, finishes transcript session, closes caption process, and closes placement helper.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- `npm run tauri dev`
- Confirm tray icon appears.
- Select a source.
- Press `Ctrl+Shift+C` and confirm captions start/stop.
- While captioning, press `Ctrl+Shift+O` and confirm overlay hides/shows while capture state remains active.
- Press `Ctrl+Shift+X` and confirm click-through toggles on the running overlay.
- Use tray `Settings` and confirm main window opens Settings.
- Use tray `Quit` and confirm no caption child process remains.

### Next

Continue with `P2-017: Performance Presets`.

### Completed: P2-017 Performance Presets

Status: implemented.

Files changed:

- `src-tauri/src/performance_settings.rs`
- `src-tauri/src/app_state.rs`
- `src-tauri/src/asr.rs`
- `src-tauri/src/audio_capture.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src/lib/domain/performance-settings.ts`
- `src/lib/tauri/commands.ts`
- `src/lib/components/overlay-settings-panel.svelte`
- `docs/WORK_LOG.md`

What changed:

- Added Rust-owned `performance-settings.json`.
- Added presets:
  - Low Resource
  - Balanced
  - Accuracy
  - Custom
- Balanced preserves the previous hardcoded ASR/VAD behavior.
- Presets now drive:
  - ASR minimum chunk size
  - ASR max buffered chunk size
  - ASR transcription interval
  - ASR worker queue capacity
  - VAD speech/silence thresholds and frame counts
- Added typed IPC:
  - `get_performance_settings`
  - `save_performance_settings`
- Added a compact Settings -> Performance section with resource impact text and custom tuning controls.
- Added unit coverage for default runtime mapping and custom clamp behavior.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- `npm run tauri dev`
- Open Settings.
- Switch Performance between Low Resource, Balanced, Accuracy, and Custom.
- Confirm the setting autosaves.
- Select a microphone or Entire System source.
- Start captions and confirm capture/ASR still starts.
- Compare caption update frequency between Low Resource and Accuracy using clear speech.

### Next

Continue with `P2-018: Error Handling And Recovery Pass`.

### Completed: P2-018 Error Handling And Recovery Pass

Status: implemented.

Files changed:

- `src-tauri/src/app_error.rs`
- `src-tauri/src/audio_capture.rs`
- `docs/WORK_LOG.md`

What changed:

- Improved Rust command error messages with user-safe recovery guidance for:
  - missing Whisper executable
  - missing ASR model file
  - ASR failures
  - unavailable audio devices
  - unsupported platform capture
  - local app data access failures
  - caption window failures
- Audio meter error events now emit user-safe recovery text instead of raw backend error strings.
- Backend command errors now log structured stderr lines with an error kind, for example `feelsay_error kind=audio ...`.
- Added unit coverage for ASR model and audio device recovery messages.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- `npm run tauri dev`
- Temporarily point Settings -> Model at a missing model path and run `Test model`.
- Confirm the UI shows a Settings -> Model recovery message.
- Select or disconnect an unavailable audio device if possible.
- Confirm the meter/error text tells the user to select a different source or reconnect the device.

### Next

Continue with `P2-019: Packaging Polish`.

### Completed: P2-019 Packaging Polish

Status: implemented.

Files changed:

- `src-tauri/tauri.conf.json`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src/lib/tauri/commands.ts`
- `src/lib/components/overlay-settings-panel.svelte`
- `docs/installer-smoke-test.md`
- `README.md`
- `docs/WORK_LOG.md`

What changed:

- Standardized packaged product casing to `FeelSay`.
- Added typed app metadata IPC:
  - `get_app_metadata`
- Settings now shows `FeelSay 0.1.0` from Rust/build metadata.
- Confirmed existing app icon assets are wired through Tauri bundle config.
- Added `docs/installer-smoke-test.md` with:
  - MSI/NSIS output paths
  - product metadata
  - installer smoke-test checklist
  - code-signing notes
- Linked the installer smoke-test doc from `README.md`.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Build outputs verified:

- `src-tauri/target/release/bundle/msi/FeelSay_0.1.0_x64_en-US.msi`
- `src-tauri/target/release/bundle/nsis/FeelSay_0.1.0_x64-setup.exe`

Manual test needed:

- Install the NSIS build on Windows.
- Confirm installed app name, icon, Settings version display, Start/Stop overlay flow, tray Quit, and uninstall behavior.

### Next

Continue with `P3-020: Speaker Or Source Labels`.

### Support: Translation Readiness Hardening

Status: implemented.

Files changed:

- `src-tauri/src/app_error.rs`
- `src-tauri/src/app_state.rs`
- `src-tauri/src/model_settings.rs`
- `src/lib/domain/model-settings.ts`
- `src/routes/+page.svelte`
- `docs/WORK_LOG.md`

What changed:

- Added model metadata field `supportsTranslation`.
- English-only Whisper models are detected from model language/name/path, including `.en.` GGML filenames.
- Translation and original-plus-translation modes now fail early with a user-safe recovery message if the active model is English-only.
- The main mode selector now warns when translation needs a multilingual model.
- Installing the default ASR assets now normalizes old English-only default slots to `Whisper Base Multilingual`.
- Added unit coverage for English-only vs multilingual model translation support detection.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test still needed:

- Open Settings -> Model.
- Install the default ASR assets to get `ggml-base.bin`, or select another multilingual GGML model.
- Set mode to `Translate to English`.
- Play or speak non-English audio.
- Confirm the overlay shows English translated captions.
- Set mode to `Original + English` and confirm two-line display.

### Next

Install/select a multilingual model and verify `P2-012` with non-English audio, or continue to `P3-020: Speaker Or Source Labels` if the MVP boundary is accepted without that manual translation verification.

### Completed: P2-012 Translation Mode MVP

Status: completed from local non-English diagnostic evidence.

Files changed:

- `src-tauri/src/asr.rs`
- `src-tauri/src/main.rs`
- `docs/WORK_LOG.md`

Local machine state changed:

- Downloaded multilingual `ggml-base.bin` to:
  - `%APPDATA%\app.feelsay.desktop\models\ggml-base.bin`
- Updated local app data `model-settings.json` so the active ASR model is:
  - `Whisper Base Multilingual`
- Reset local app data `caption-settings.json` back to:
  - `captions`
- Downloaded a temporary CC0 Spanish WAV sample from Wikimedia Commons into `%TEMP%` for verification only.

What changed:

- Added CLI diagnostic:
  - `cargo run --manifest-path src-tauri/Cargo.toml -- --asr-file-diagnostic <wav>`
- The diagnostic uses Feelsay's configured ASR runtime and current caption mode against a provided WAV file.
- Updated the external `whisper.cpp` wrapper to pass `-l auto`, so original transcription auto-detects the source language.
- This fixes original-plus-English mode for non-English input.

Verification:

- Translation mode with Spanish WAV:
  - input meaning: `¿Puedes ayudarme?`
  - output: `Can you help me?`
- Original-plus-English mode with Spanish WAV:
  - output line 1: `¿Puedes ayudarme?`
  - output line 2: `Can you help me?`

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual app smoke test still useful:

- `npm run tauri dev`
- Set mode to `Translate to English`.
- Play non-English speech through system audio.
- Confirm the running overlay shows English captions.
- Set mode to `Original + English`.
- Confirm the running overlay shows original text above English translation.

### Next

MVP backlog through `P2-019` is implemented and verified by automated checks plus local diagnostics. Future work starts at `P3-020: Speaker Or Source Labels`.

### Completed: P3-020 Speaker Or Source Labels

Status: implemented.

Files changed:

- `src-tauri/src/app_state.rs`
- `src-tauri/src/asr.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/main.rs`
- `src-tauri/src/transcript.rs`
- `src/lib/tauri/commands.ts`
- `src/routes/+page.svelte`
- `docs/WORK_LOG.md`

What changed:

- The selected source display label is passed from the main UI to `start_audio_meter`.
- `AsrRuntimeConfig` now carries an optional normalized `source_label`.
- Caption runtime JSON now includes `sourceLabel`, so the caption child process can display the active source.
- The Windows caption overlay draws the source label as a smaller line above caption text when present.
- Transcript segments now receive the same source label instead of `None`.
- TXT/SRT/VTT transcript exports include labels in the form `[Source] text`.
- Added focused tests for caption runtime source labels and transcript export labels.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Note:

- An initial parallel `npm run build` run failed because it raced with `npm run tauri build` over `.svelte-kit`; rerunning `npm run build` by itself passed.

Manual test needed:

- Start captions from a microphone or system audio source.
- Confirm the overlay shows a small source label above caption text.
- Enable transcript saving, speak/play audio, stop captions, export TXT/SRT/VTT, and confirm exported segments include the source label.

### Next

Continue with `P3-021: Diarization` only after deciding the MVP should include speaker-turn detection work now.

### Completed: P3-021 Diarization

Status: implemented as an opt-in experimental scaffold.

Files changed:

- `src-tauri/src/diarization.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/app_state.rs`
- `src-tauri/src/asr.rs`
- `src-tauri/src/caption_settings.rs`
- `src-tauri/src/main.rs`
- `src-tauri/src/transcript.rs`
- `src/lib/components/overlay-settings-panel.svelte`
- `src/lib/domain/caption-settings.ts`
- `src/lib/domain/transcript-settings.ts`
- `src/routes/+page.svelte`
- `docs/diarization-evaluation.md`
- `docs/WORK_LOG.md`

What changed:

- Added Rust `diarization` module with an opt-in local experimental speaker labeler.
- Added caption setting `speakerLabelsEnabled`, defaulting to off.
- Added Settings checkbox: `Experimental speaker labels`.
- ASR runtime now carries the speaker-label setting and generates non-blocking speaker labels when enabled.
- Caption runtime JSON now includes `speakerLabel` and `speakerConfidence`.
- Caption overlay combines source and speaker labels, including uncertainty.
- Transcript segments now store `speaker_label` and `speaker_confidence`.
- Existing transcript databases migrate with added nullable columns.
- TXT/SRT/VTT transcript exports include source and speaker labels.
- Added diarization evaluation notes for pyannote.audio, WhisperX, and WeSpeaker.

Limitations:

- This is not production diarization.
- The current labeler is a lightweight local heuristic with only `low` or `medium` confidence.
- Real speaker embeddings/model-based diarization should be added later as an optional high-resource worker.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- Open Settings.
- Enable `Experimental speaker labels`.
- Start captions from a microphone source.
- Confirm the overlay can show a speaker label with confidence.
- Enable transcript saving, stop captions, export TXT/SRT/VTT, and confirm speaker labels are included.

### Next

Continue with `P3-022: Local Text Translation Engine`.

### Completed: P3-022 Local Text Translation Engine

Status: implemented with a local Argos CLI adapter.

Files changed:

- `src-tauri/src/app_error.rs`
- `src-tauri/src/app_state.rs`
- `src-tauri/src/asr.rs`
- `src-tauri/src/caption_settings.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/translation.rs`
- `src/lib/components/overlay-settings-panel.svelte`
- `src/lib/domain/caption-settings.ts`
- `src/lib/domain/translation-settings.ts`
- `src/lib/tauri/commands.ts`
- `src/routes/+page.svelte`
- `docs/local-translation-engine.md`
- `docs/WORK_LOG.md`

What changed:

- Added Rust-owned `translation-settings.json` for the local text translation engine.
- Added `TranslationLanguage`, `TranslationSettings`, engine status, and diagnostic types.
- Added Argos Translate CLI adapter for offline text translation.
- Added IPC for translation settings, engine status, and translation diagnostics.
- Added source and target language fields to caption settings.
- Preserved Whisper's built-in English translation path.
- Added non-English target translation after ASR through the separate translation adapter.
- Added Settings -> Translation controls for source language, target language, Argos executable, and test translation.
- Updated main caption mode labels to reflect the selected target language.
- Documented local translation setup and evaluated engine options in `docs/local-translation-engine.md`.

Limitations:

- Feelsay does not bundle Argos Translate, Python, or language packages.
- Real local translation requires `argos-translate` and the needed language pair installed locally.
- Text translation source language must be selected for non-English target translation.
- Model/package install management remains external in this PBI.

Checks passed:

- `npm run format`
- `npm run format:check`
- `npm run lint`
- `npm run rust:lint`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run tauri build`

Manual test needed:

- Install Argos Translate and a language package, for example `translate-en_es`.
- Open Settings -> Translation.
- Set `From` and `To`.
- Click `Test translation`.
- Select a source, choose `Translate to <target>` or `Original + <target>`, and start captions.

### Next

Continue with `P3-023: Custom Vocabulary And Corrections`.
