<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import type {
    OverlayProfileId,
    OverlayPlacement,
    OverlaySettings,
    OverlaySettingsProfile,
    OverlaySettingsStore,
  } from "$lib/domain/overlay-settings";
  import {
    colorPickerValue,
    hasValidOverlaySettingsColors,
    normalizeCssColor,
    rgbaPreviewColor,
  } from "$lib/domain/overlay-settings";
  import {
    commandErrorMessage,
    getOverlayPlacement,
    getOverlaySettingsStore,
    saveOverlaySettingsStore,
    startOverlayPlacement,
    stopOverlayPlacement,
  } from "$lib/tauri/commands";

  type Props = {
    onClose: () => void;
  };

  type SettingsState =
    | { status: "loading" }
    | { status: "ready" }
    | { status: "error"; message: string };

  type SaveState =
    | { status: "saved" }
    | { status: "saving" }
    | { status: "error"; message: string };

  const fontOptions = [
    "Segoe UI",
    "Arial",
    "Calibri",
    "Verdana",
    "Tahoma",
    "Trebuchet MS",
    "Georgia",
    "Times New Roman",
    "Courier New",
    "Consolas",
  ];

  let { onClose }: Props = $props();
  let settingsState = $state<SettingsState>({ status: "loading" });
  let settingsStore = $state<OverlaySettingsStore | null>(null);
  let saveState = $state<SaveState>({ status: "saved" });
  let placement = $state<OverlayPlacement | null>(null);
  let placementOrigin = $state<OverlayPlacement | null>(null);
  let placementState = $state<"idle" | "starting" | "active" | "error">("idle");
  let placementMessage = $state("");
  let placementPoll: ReturnType<typeof setInterval> | undefined;
  let autosaveTimer: ReturnType<typeof setTimeout> | undefined;
  let lastSavedStoreJson = "";
  let saveRequestId = 0;

  onMount(() => {
    void loadSettings();
  });

  onDestroy(() => {
    stopAutosaveTimer();
    void stopPlacement();
  });

  $effect(() => {
    const store = settingsStore;

    if (!store || settingsState.status !== "ready") {
      return;
    }

    const serializedStore = serializeStore(store);

    if (serializedStore === lastSavedStoreJson) {
      return;
    }

    if (!storeHasValidColors(store)) {
      stopAutosaveTimer();
      saveRequestId += 1;
      saveState = { status: "error", message: "Check colors" };
      return;
    }

    saveState = { status: "saving" };
    stopAutosaveTimer();

    const requestId = ++saveRequestId;
    autosaveTimer = setTimeout(() => {
      void autosaveSettings(store, requestId);
    }, 200);
  });

  async function loadSettings() {
    try {
      settingsStore = await getOverlaySettingsStore();
      lastSavedStoreJson = serializeStore(settingsStore);
      saveState = { status: "saved" };
      settingsState = { status: "ready" };
    } catch (error) {
      settingsState = {
        status: "error",
        message: commandErrorMessage(error),
      };
    }
  }

  async function autosaveSettings(
    storeSnapshot: OverlaySettingsStore,
    requestId: number,
  ) {
    try {
      const savedStore = await saveOverlaySettingsStore(storeSnapshot);

      if (requestId !== saveRequestId) {
        return;
      }

      lastSavedStoreJson = serializeStore(savedStore);
      settingsStore = savedStore;
      saveState = { status: "saved" };
    } catch (error) {
      if (requestId !== saveRequestId) {
        return;
      }

      saveState = {
        status: "error",
        message: commandErrorMessage(error),
      };
    }
  }

  function stopAutosaveTimer() {
    if (autosaveTimer) {
      clearTimeout(autosaveTimer);
      autosaveTimer = undefined;
    }
  }

  function serializeStore(store: OverlaySettingsStore): string {
    return JSON.stringify(store);
  }

  function storeHasValidColors(store: OverlaySettingsStore): boolean {
    return store.profiles.every((profile) =>
      hasValidOverlaySettingsColors(profile.settings),
    );
  }

  function getActiveProfile(
    store: OverlaySettingsStore,
  ): OverlaySettingsProfile {
    return (
      store.profiles.find((profile) => profile.id === store.activeProfileId) ??
      store.profiles[0]
    );
  }

  function getActiveSettings(): OverlaySettings | null {
    if (!settingsStore) {
      return null;
    }

    return getActiveProfile(settingsStore).settings;
  }

  function selectProfile(profileId: OverlayProfileId) {
    if (!settingsStore) {
      return;
    }

    settingsStore = {
      ...settingsStore,
      activeProfileId: profileId,
    };
  }

  function updateProfileName(name: string) {
    const store = settingsStore;

    if (!store) {
      return;
    }

    settingsStore = {
      ...store,
      profiles: store.profiles.map((profile) =>
        profile.id === store.activeProfileId ? { ...profile, name } : profile,
      ),
    };
  }

  async function beginPlacement() {
    const settings = getActiveSettings();

    if (!settings) {
      return;
    }

    placementState = "starting";
    placementMessage = "";
    placementOrigin = draftPlacement(settings);

    try {
      applyPlacement(await startOverlayPlacement(settings));
      placementState = "active";
      startPlacementPolling();
    } catch (error) {
      placementState = "error";
      placementMessage = commandErrorMessage(error);
    }
  }

  async function refreshPlacement() {
    if (placementState !== "active") {
      return;
    }

    try {
      const nextPlacement = await getOverlayPlacement();

      if (nextPlacement) {
        applyPlacement(nextPlacement);

        if (nextPlacement.isAccepted) {
          await finishPlacement(false);
        }

        return;
      }

      await finishPlacement(true);
    } catch (error) {
      stopPlacementPolling();
      placementState = "error";
      placementMessage = commandErrorMessage(error);
    }
  }

  function startPlacementPolling() {
    stopPlacementPolling();
    placementPoll = setInterval(() => {
      void refreshPlacement();
    }, 80);
  }

  function stopPlacementPolling() {
    if (placementPoll) {
      clearInterval(placementPoll);
      placementPoll = undefined;
    }
  }

  async function usePlacement() {
    await finishPlacement(false);
  }

  async function cancelPlacement() {
    await finishPlacement(true);
  }

  async function finishPlacement(restoreOriginal: boolean) {
    stopPlacementPolling();

    if (restoreOriginal && placementOrigin) {
      applyPlacement(placementOrigin);
    }

    placementState = "idle";
    placementMessage = "";
    placement = null;
    placementOrigin = null;

    try {
      await stopOverlayPlacement();
    } catch {
      // The helper may already be closed by the user.
    }
  }

  async function stopPlacement() {
    await finishPlacement(false);
  }

  function applyPlacement(nextPlacement: OverlayPlacement) {
    placement = nextPlacement;

    updateActiveSettings({
      startX: nextPlacement.x,
      startY: nextPlacement.y,
      startWidth: nextPlacement.width,
      startHeight: nextPlacement.height,
    });
  }

  function draftPlacement(settings: OverlaySettings): OverlayPlacement {
    return {
      x: settings.startX ?? 0,
      y: settings.startY ?? 0,
      width: settings.startWidth,
      height: settings.startHeight,
      isAccepted: false,
    };
  }

  function updateSetting<Key extends keyof OverlaySettings>(
    key: Key,
    value: OverlaySettings[Key],
  ) {
    updateActiveSettings({ [key]: value } as Pick<OverlaySettings, Key>);
  }

  function updateActiveSettings(settingsPatch: Partial<OverlaySettings>) {
    const store = settingsStore;

    if (!store) {
      return;
    }

    settingsStore = {
      ...store,
      profiles: store.profiles.map((profile) =>
        profile.id === store.activeProfileId
          ? {
              ...profile,
              settings: {
                ...profile.settings,
                ...settingsPatch,
              },
            }
          : profile,
      ),
    };
  }

  function textInput(event: Event): string {
    return (event.currentTarget as HTMLInputElement).value;
  }

  function numberInput(event: Event): number {
    return Number((event.currentTarget as HTMLInputElement).value);
  }

  function optionalNumberInput(event: Event): number | null {
    const value = (event.currentTarget as HTMLInputElement).value.trim();
    return value.length > 0 ? Number(value) : null;
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      void closeSettings();
    }
  }

  async function closeSettings() {
    await stopPlacement();
    onClose();
  }

  function previewSurfaceStyle(settings: OverlaySettings): string {
    return `background: ${rgbaPreviewColor(settings.backgroundColor, settings.backgroundOpacity)}`;
  }

  function previewTextStyle(settings: OverlaySettings): string {
    const textColor = normalizeCssColor(settings.textColor, "#ffffff");
    const outlineColor = normalizeCssColor(settings.outlineColor, "#000000");
    const fontWeight = settings.fontWeight === "bold" ? 700 : 400;

    return [
      `font-family: ${JSON.stringify(settings.fontFamily)}, sans-serif`,
      `font-size: ${settings.fontSize}px`,
      `font-weight: ${fontWeight}`,
      `color: ${textColor}`,
      `-webkit-text-stroke: ${settings.outlineWidth}px ${outlineColor}`,
    ].join(";");
  }

  function opacityPercent(value: number): number {
    return Math.round(Math.min(Math.max(value, 0), 1) * 100);
  }

  function saveStatusText(state: SaveState): string {
    if (state.status === "saving") {
      return "Saving";
    }

    if (state.status === "error") {
      return state.message;
    }

    return "Saved";
  }
</script>

<div
  class="settings-backdrop"
  role="presentation"
  onclick={handleBackdropClick}
>
  <div
    class="settings-panel"
    role="dialog"
    aria-modal="true"
    aria-labelledby="overlay-settings-title"
  >
    <header class="settings-header">
      <h2 id="overlay-settings-title">Settings</h2>
      <div class="settings-header-actions">
        <div
          class:save-error={saveState.status === "error"}
          class="save-status"
        >
          <span class="save-status-icon" aria-hidden="true">
            {saveState.status === "saved" ? "✓" : ""}
          </span>
          <span>{saveStatusText(saveState)}</span>
        </div>
        <button
          type="button"
          aria-label="Close settings"
          onclick={() => void closeSettings()}
        >
          Close
        </button>
      </div>
    </header>

    {#if settingsState.status === "loading"}
      <p class="settings-message">Loading</p>
    {:else if settingsState.status === "error"}
      <p class="settings-message" role="alert">{settingsState.message}</p>
    {:else if settingsStore}
      {@const profile = getActiveProfile(settingsStore)}
      {@const draft = profile.settings}
      <div class="profile-bar" aria-label="Overlay profiles">
        <div class="profile-tabs">
          {#each settingsStore.profiles as profileOption}
            <button
              class:active-profile={profileOption.id ===
                settingsStore.activeProfileId}
              type="button"
              onclick={() => selectProfile(profileOption.id)}
            >
              {profileOption.name}
            </button>
          {/each}
        </div>
        <label class="profile-name-field">
          Profile name
          <input
            type="text"
            maxlength="32"
            value={profile.name}
            oninput={(event) => updateProfileName(textInput(event))}
          />
        </label>
      </div>

      <div class="settings-grid">
        <form
          class="settings-form"
          onsubmit={(event) => event.preventDefault()}
        >
          <fieldset>
            <legend>Text</legend>
            <label>
              Font
              <select
                value={draft.fontFamily}
                onchange={(event) =>
                  updateSetting("fontFamily", textInput(event))}
              >
                {#if !fontOptions.includes(draft.fontFamily)}
                  <option value={draft.fontFamily}>{draft.fontFamily}</option>
                {/if}
                {#each fontOptions as fontOption}
                  <option value={fontOption}>{fontOption}</option>
                {/each}
              </select>
            </label>
            <label>
              Size
              <input
                type="number"
                min="18"
                max="96"
                value={draft.fontSize}
                oninput={(event) =>
                  updateSetting("fontSize", numberInput(event))}
              />
            </label>
            <label>
              Weight
              <select
                value={draft.fontWeight}
                onchange={(event) =>
                  updateSetting(
                    "fontWeight",
                    textInput(event) === "bold" ? "bold" : "normal",
                  )}
              >
                <option value="normal">Normal</option>
                <option value="bold">Bold</option>
              </select>
            </label>
            <label>
              Color
              <span class="color-row">
                <input
                  aria-label="Text color picker"
                  type="color"
                  value={colorPickerValue(draft.textColor)}
                  oninput={(event) =>
                    updateSetting("textColor", textInput(event))}
                />
                <input
                  type="text"
                  value={draft.textColor}
                  oninput={(event) =>
                    updateSetting("textColor", textInput(event))}
                />
              </span>
            </label>
          </fieldset>

          <fieldset>
            <legend>Outline</legend>
            <label>
              Width
              <input
                type="number"
                min="0"
                max="8"
                value={draft.outlineWidth}
                oninput={(event) =>
                  updateSetting("outlineWidth", numberInput(event))}
              />
            </label>
            <label>
              Color
              <span class="color-row">
                <input
                  aria-label="Outline color picker"
                  type="color"
                  value={colorPickerValue(draft.outlineColor, "#000000")}
                  oninput={(event) =>
                    updateSetting("outlineColor", textInput(event))}
                />
                <input
                  type="text"
                  value={draft.outlineColor}
                  oninput={(event) =>
                    updateSetting("outlineColor", textInput(event))}
                />
              </span>
            </label>
          </fieldset>

          <fieldset>
            <legend>Background</legend>
            <label>
              Color
              <span class="color-row">
                <input
                  aria-label="Background color picker"
                  type="color"
                  value={colorPickerValue(draft.backgroundColor, "#000000")}
                  oninput={(event) =>
                    updateSetting("backgroundColor", textInput(event))}
                />
                <input
                  type="text"
                  value={draft.backgroundColor}
                  oninput={(event) =>
                    updateSetting("backgroundColor", textInput(event))}
                />
              </span>
            </label>
            <label>
              Opacity
              <span class="range-row">
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.01"
                  value={draft.backgroundOpacity}
                  oninput={(event) =>
                    updateSetting("backgroundOpacity", numberInput(event))}
                />
                <output>{opacityPercent(draft.backgroundOpacity)}%</output>
              </span>
            </label>
          </fieldset>

          <fieldset>
            <legend>Interaction</legend>
            <label class="checkbox-row">
              <input
                type="checkbox"
                checked={draft.clickThrough}
                onchange={(event) =>
                  updateSetting(
                    "clickThrough",
                    (event.currentTarget as HTMLInputElement).checked,
                  )}
              />
              <span>
                <strong>Click-through overlay</strong>
                <small>Let clicks pass through the caption box.</small>
              </span>
            </label>
          </fieldset>

          <fieldset>
            <legend>Starting window</legend>
            <div class="two-column-fields">
              <label>
                Width
                <input
                  type="number"
                  min="300"
                  max="1800"
                  value={draft.startWidth}
                  oninput={(event) =>
                    updateSetting("startWidth", numberInput(event))}
                />
              </label>
              <label>
                Height
                <input
                  type="number"
                  min="80"
                  max="600"
                  value={draft.startHeight}
                  oninput={(event) =>
                    updateSetting("startHeight", numberInput(event))}
                />
              </label>
              <label>
                X
                <input
                  type="number"
                  value={draft.startX ?? ""}
                  oninput={(event) =>
                    updateSetting("startX", optionalNumberInput(event))}
                />
              </label>
              <label>
                Y
                <input
                  type="number"
                  value={draft.startY ?? ""}
                  oninput={(event) =>
                    updateSetting("startY", optionalNumberInput(event))}
                />
              </label>
            </div>

            <div class="placement-tools">
              <button
                type="button"
                class="secondary-action"
                disabled={placementState === "starting"}
                onclick={beginPlacement}
              >
                {placementState === "starting"
                  ? "Opening"
                  : "Position visually"}
              </button>

              {#if placementState === "active" && placement}
                <div class="placement-readout" aria-live="polite">
                  <span>X {placement.x}</span>
                  <span>Y {placement.y}</span>
                  <span>W {placement.width}</span>
                  <span>H {placement.height}</span>
                </div>
                <div class="placement-actions">
                  <button
                    type="button"
                    class="secondary-action"
                    onclick={() => void cancelPlacement()}
                  >
                    Cancel placement
                  </button>
                  <button
                    type="button"
                    class="primary-action"
                    onclick={() => void usePlacement()}
                  >
                    Use placement
                  </button>
                </div>
              {:else if placementState === "error"}
                <p class="placement-error" role="alert">{placementMessage}</p>
              {/if}
            </div>
          </fieldset>
        </form>

        <aside class="preview-pane" aria-label="Overlay preview">
          <div class="preview-stage">
            <div class="caption-preview" style={previewSurfaceStyle(draft)}>
              <p style={previewTextStyle(draft)}>
                Live captions will appear here.
              </p>
            </div>
          </div>
        </aside>
      </div>
    {/if}
  </div>
</div>

<style>
  .settings-backdrop {
    position: fixed;
    inset: 0;
    z-index: 20;
    display: grid;
    place-items: center;
    padding: 24px;
    background: oklch(0% 0 0 / 0.58);
  }

  .settings-panel {
    display: grid;
    grid-template-rows: auto auto minmax(0, 1fr);
    box-sizing: border-box;
    width: min(960px, 100%);
    max-height: min(760px, calc(100vh - 48px));
    overflow: hidden;
    border: 1px solid var(--borderColor-default);
    border-radius: var(--radius-default);
    background: var(--bgColor-raised);
    box-shadow: var(--shadow-soft);
  }

  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 16px 18px;
  }

  .settings-header {
    border-bottom: 1px solid var(--borderColor-muted);
  }

  .settings-header-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .save-status {
    display: flex;
    align-items: center;
    gap: 7px;
    color: var(--fgColor-muted);
    font-size: 0.82rem;
    font-weight: 700;
  }

  .save-status-icon {
    display: grid;
    width: 16px;
    height: 16px;
    place-items: center;
    border-radius: 999px;
    background: var(--button-primary-bgColor-rest);
    color: var(--fgColor-onEmphasis);
    font-size: 0.7rem;
    line-height: 1;
  }

  .save-status:not(.save-error) .save-status-icon:empty {
    background: transparent;
    border: 2px solid var(--button-primary-bgColor-rest);
  }

  .save-status.save-error {
    color: var(--danger-fgColor, oklch(74% 0.16 24));
  }

  .save-status.save-error .save-status-icon {
    background: var(--danger-fgColor, oklch(74% 0.16 24));
  }

  .profile-bar {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(180px, 240px);
    gap: 12px;
    align-items: end;
    min-width: 0;
    padding: 14px 18px;
    border-bottom: 1px solid var(--borderColor-muted);
  }

  .profile-tabs {
    display: flex;
    gap: 8px;
    min-width: 0;
    overflow-x: auto;
    scrollbar-width: none;
  }

  .profile-tabs::-webkit-scrollbar {
    display: none;
  }

  .profile-tabs button {
    min-height: 34px;
    max-width: 128px;
    flex: 0 0 auto;
    overflow: hidden;
    border: 1px solid var(--borderColor-default);
    border-radius: var(--radius-default);
    padding: 0 12px;
    background: transparent;
    color: var(--fgColor-muted);
    cursor: pointer;
    font-weight: 700;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .profile-tabs button:hover,
  .profile-tabs button.active-profile {
    background: var(--button-secondary-bgColor-hover);
    color: var(--fgColor-default);
  }

  .profile-tabs button.active-profile {
    border-color: var(--button-primary-bgColor-rest);
  }

  .profile-name-field {
    gap: 5px;
  }

  h2 {
    margin: 0;
    color: var(--fgColor-default);
    font-size: 1.125rem;
  }

  .settings-header button,
  .secondary-action,
  .primary-action {
    min-height: 36px;
    border-radius: var(--radius-default);
    padding: 0 14px;
    font-weight: 700;
    cursor: pointer;
  }

  .settings-header button,
  .secondary-action {
    border: 1px solid var(--borderColor-default);
    background: transparent;
    color: var(--fgColor-default);
  }

  .settings-header button:hover,
  .secondary-action:hover {
    background: var(--button-secondary-bgColor-hover);
  }

  .secondary-action:disabled {
    cursor: default;
    opacity: 0.55;
  }

  .settings-grid {
    display: grid;
    grid-template-columns: minmax(340px, 400px) minmax(0, 1fr);
    min-height: 0;
    min-width: 0;
    overflow: hidden;
  }

  .settings-form {
    display: grid;
    align-content: start;
    gap: 16px;
    min-width: 0;
    box-sizing: border-box;
    overflow: auto;
    padding: 18px;
    scrollbar-width: none;
  }

  .settings-form::-webkit-scrollbar {
    display: none;
  }

  fieldset {
    display: grid;
    gap: 12px;
    min-width: 0;
    margin: 0;
    border: 0;
    padding: 0;
  }

  legend {
    margin-bottom: 2px;
    color: var(--fgColor-default);
    font-weight: 800;
  }

  label {
    display: grid;
    gap: 6px;
    min-width: 0;
    color: var(--fgColor-muted);
    font-size: 0.875rem;
  }

  output {
    color: var(--fgColor-default);
    font-size: 0.8rem;
    font-weight: 700;
  }

  input,
  select {
    width: 100%;
    min-width: 0;
    min-height: 36px;
    box-sizing: border-box;
    border: 1px solid var(--borderColor-default);
    border-radius: var(--radius-default);
    padding: 0 10px;
    background: var(--bgColor-inset);
    color: var(--fgColor-default);
    font: inherit;
  }

  input[type="color"] {
    width: 44px;
    padding: 3px;
    cursor: pointer;
  }

  input[type="range"] {
    padding: 0;
    cursor: pointer;
  }

  input[type="checkbox"] {
    width: 18px;
    min-height: 18px;
    accent-color: var(--button-primary-bgColor-rest);
    cursor: pointer;
  }

  .range-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 44px;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }

  .range-row input {
    min-height: 24px;
  }

  output {
    text-align: right;
  }

  .color-row {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 8px;
    min-width: 0;
  }

  .two-column-fields {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    min-width: 0;
  }

  .checkbox-row {
    grid-template-columns: auto minmax(0, 1fr);
    align-items: start;
    gap: 10px;
    color: var(--fgColor-default);
    cursor: pointer;
  }

  .checkbox-row span {
    display: grid;
    gap: 2px;
  }

  .checkbox-row small {
    color: var(--fgColor-muted);
    font-size: 0.8rem;
  }

  .placement-tools {
    display: grid;
    gap: 10px;
  }

  .placement-readout {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 6px;
    color: var(--fgColor-muted);
    font-size: 0.8rem;
  }

  .placement-readout span {
    border: 1px solid var(--borderColor-muted);
    border-radius: var(--radius-default);
    padding: 6px 8px;
    background: var(--bgColor-inset);
  }

  .placement-actions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }

  .placement-error {
    margin: 0;
    color: var(--danger-fgColor, oklch(74% 0.16 24));
    font-size: 0.84rem;
  }

  .preview-pane {
    display: grid;
    min-width: 0;
    box-sizing: border-box;
    border-left: 1px solid var(--borderColor-muted);
    padding: 18px;
  }

  .preview-stage {
    display: grid;
    min-height: 280px;
    place-items: center;
    border-radius: var(--radius-default);
    background:
      linear-gradient(45deg, oklch(100% 0 0 / 0.08) 25%, transparent 25%),
      linear-gradient(-45deg, oklch(100% 0 0 / 0.08) 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, oklch(100% 0 0 / 0.08) 75%),
      linear-gradient(-45deg, transparent 75%, oklch(100% 0 0 / 0.08) 75%),
      oklch(18% 0.016 245);
    background-position:
      0 0,
      0 10px,
      10px -10px,
      -10px 0;
    background-size: 20px 20px;
  }

  .caption-preview {
    display: grid;
    width: min(460px, 100%);
    min-height: 92px;
    place-items: center;
    padding: 16px;
    resize: both;
    overflow: hidden;
  }

  .caption-preview p {
    margin: 0;
    text-align: center;
    line-height: 1.2;
  }

  .primary-action {
    border: 1px solid transparent;
    background: var(--button-primary-bgColor-rest);
    color: var(--fgColor-onEmphasis);
  }

  .primary-action:not(:disabled):hover {
    background: var(--button-primary-bgColor-hover);
  }

  .primary-action:disabled {
    cursor: default;
    opacity: 0.55;
  }

  .settings-message {
    margin: 0;
    padding: 18px;
    color: var(--fgColor-muted);
  }

  @media (max-width: 900px) {
    .settings-panel {
      max-height: min(760px, calc(100vh - 32px));
    }

    .settings-grid {
      grid-template-columns: 1fr;
      overflow: auto;
      scrollbar-width: none;
    }

    .profile-bar {
      grid-template-columns: 1fr;
    }

    .settings-grid::-webkit-scrollbar {
      display: none;
    }

    .settings-form {
      overflow: visible;
    }

    .preview-pane {
      border-top: 1px solid var(--borderColor-muted);
      border-left: 0;
    }
  }
</style>
