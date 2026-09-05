# Caption Overlay

The Caption Overlay is the floating subtitle window that appears when the user starts captions.

It is intentionally simple: it displays caption text only. It should not contain dashboard controls, source selection, settings menus, or other normal in-overlay UI.

## Current Implementation

- Windows-only child process launched by the main Tauri app.
- Rendered with Win32/GDI, not a Svelte webview.
- Frameless, always-on-top, resizable, and draggable.
- Uses a transparent/translucent layered window.
- Uses a tiny nonzero alpha hit area when opacity is visually `0%` so the window can still be dragged.
- Can be set to click-through mode so clicks pass to apps behind it.
- Reads live settings from a runtime JSON file while running.

This child-process model replaced the earlier Tauri overlay window because lifecycle, native close, and drag behavior were unreliable for the frameless transparent overlay.

## Lifecycle Rules

- `Start` opens one caption child process.
- `Stop` closes the caption child process.
- Closing the Main Window closes the caption child process.
- Starting again after close creates a fresh overlay.
- There must never be duplicate overlay instances.
- There must never be orphan/zombie overlay windows.

## Settings

Overlay appearance and placement are owned by Rust settings and exposed in the Settings modal.

Supported settings:

- Font family
- Font size
- Font weight
- Text color
- Outline color
- Outline width
- Background color
- Background opacity
- Starting width
- Starting height
- Starting X/Y position
- Click-through mode
- Five named profiles

Settings autosave and live-update the running overlay.

## Visual Placement

The visual placement helper is a temporary child-process window used only from Settings.

It lets the user drag and resize a preview overlay, then accept that geometry into the current profile. It is intentionally separate from the real caption overlay and remains interactive even if the real overlay profile has click-through enabled.

## Important Constraints

- Keep the normal Caption Overlay text-only.
- Keep child-process lifecycle simple.
- Do not add Svelte/Tauri webview controls inside the overlay unless there is a strong reason.
- Keep placement/configuration in the Settings flow.
- Keep ASR, translation, and transcript logic outside the overlay window.
