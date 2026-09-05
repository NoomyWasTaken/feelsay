# Application Audio Capture

## Current Decision

Application/window sources are enumerated for UX discovery, but app-specific audio capture is not enabled yet.

The MVP capture path remains:

- Microphone capture through WASAPI input devices.
- Entire system/output-device capture through WASAPI loopback.
- App/window sources visible but marked unsupported.

## Windows Implementation Path

Windows has a process-loopback capture API through `ActivateAudioInterfaceAsync` with `AUDIOCLIENT_ACTIVATION_PARAMS` and `PROCESS_LOOPBACK_MODE`.

Official references:

- Microsoft sample: https://learn.microsoft.com/en-us/samples/microsoft/windows-classic-samples/applicationloopbackaudio-sample/
- `PROCESS_LOOPBACK_MODE`: https://learn.microsoft.com/en-us/windows/win32/api/audioclientactivationparams/ne-audioclientactivationparams-process_loopback_mode
- `ActivateAudioInterfaceAsync`: https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/nf-mmdeviceapi-activateaudiointerfaceasync

Expected adapter shape:

1. User selects a `window:{hwnd}:{pid}` source.
2. Rust resolves the current process id from the source metadata.
3. A Windows-only application-loopback provider starts process-tree loopback capture.
4. Captured PCM frames are normalized through the existing `PcmAudioFrame` path.
5. VAD, ASR, captions, and transcripts consume the same frame stream as microphone/system capture.

## Why It Is Not Enabled Yet

The process-loopback API is async COM-based and needs a dedicated implementation with clear lifecycle handling. It should not be mixed into the current stable system-loopback code casually.

Known constraints:

- It is Windows-only.
- It captures by process id/process tree, not a visual window handle directly.
- Some modern apps may use child processes or audio sessions that do not map cleanly to the selected top-level window.
- Some communication apps can be unreliable with app-specific loopback; system audio fallback must remain available.

## Backlog Scope

For `P2-014`, the current scoped outcome is:

- App/window enumeration remains real.
- App/window capture remains disabled and honest in the UI.
- `supportsApplicationAudio` is `false` until the Windows process-loopback adapter exists.
- The next implementation should add a dedicated Windows process-loopback provider instead of changing frontend source logic.
