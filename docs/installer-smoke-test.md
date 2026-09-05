# Installer Smoke Test

Use this checklist before sharing a Windows build.

## Build Outputs

- MSI: `src-tauri/target/release/bundle/msi/FeelSay_0.1.0_x64_en-US.msi`
- NSIS: `src-tauri/target/release/bundle/nsis/FeelSay_0.1.0_x64-setup.exe`

## Metadata

- Product name: `FeelSay`
- App identifier: `app.feelsay.desktop`
- Version: `0.1.0`
- Icon source: `src-tauri/icons/icon.png`

## Smoke Test

1. Run `npm run tauri build`.
2. Install the NSIS build on a clean Windows profile if possible.
3. Confirm Start Menu and installed app name show `FeelSay`.
4. Launch the app.
5. Open Settings and confirm the version is visible.
6. Select `Entire System` or a microphone source.
7. Start captions and confirm the caption overlay opens.
8. Stop captions and confirm the overlay closes.
9. Quit from the tray and confirm no caption child process remains.
10. Uninstall and confirm the app is removed cleanly.

## Code Signing Notes

- Current local builds are unsigned.
- Production Windows distribution should use a trusted code-signing certificate.
- Sign both MSI/NSIS output and the bundled executable as required by the release process.
- Keep private signing keys out of the repository.
