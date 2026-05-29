<script lang="ts">
  import { onMount } from "svelte";
  import type { AppSettings } from "$lib/domain/settings";
  import { commandErrorMessage } from "$lib/tauri/commands";

  type Props = {
    settings: AppSettings;
    onCancel: () => void;
    onSave: (settings: AppSettings) => Promise<void>;
    onTestCaption: (settings: AppSettings) => Promise<void>;
  };

  let { settings, onCancel, onSave, onTestCaption }: Props = $props();
  let draft = $state<AppSettings | null>(null);
  let panelState = $state<"idle" | "saving" | "testing">("idle");
  let errorMessage = $state("");

  onMount(() => {
    draft = structuredClone(settings);
  });

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onCancel();
    }
  }

  async function saveSettings() {
    const currentDraft = draft;
    if (!currentDraft) {
      return;
    }

    await runPanelAction("saving", () => onSave(currentDraft));
  }

  async function testCaption() {
    const currentDraft = draft;
    if (!currentDraft) {
      return;
    }

    await runPanelAction("testing", () => onTestCaption(currentDraft));
  }

  async function runPanelAction(
    state: "saving" | "testing",
    action: () => Promise<void>,
  ) {
    panelState = state;
    errorMessage = "";

    try {
      await action();
      if (state === "saving") {
        onCancel();
      }
    } catch (error) {
      errorMessage = commandErrorMessage(error);
    } finally {
      panelState = "idle";
    }
  }
</script>

<div class="modal-backdrop" role="presentation" onclick={handleBackdropClick}>
  <div
    class="settings-panel"
    role="dialog"
    aria-modal="true"
    aria-labelledby="settings-title"
    tabindex="-1"
  >
    <header class="panel-header">
      <h2 id="settings-title">Overlay Settings</h2>
      <button
        class="icon-button"
        type="button"
        aria-label="Close settings"
        onclick={onCancel}
      >
        x
      </button>
    </header>

    <div class="settings-body">
      {#if draft}
        <label>
          <span>Width</span>
          <input
            type="number"
            min="320"
            max="1800"
            step="10"
            bind:value={draft.overlay.width}
          />
        </label>

        <label>
          <span>Height</span>
          <input
            type="number"
            min="96"
            max="900"
            step="10"
            bind:value={draft.overlay.height}
          />
        </label>

        <label>
          <span>Overlay opacity</span>
          <input
            type="range"
            min="0"
            max="1"
            step="0.05"
            bind:value={draft.overlay.opacity}
          />
        </label>

        <label>
          <span>Background opacity</span>
          <input
            type="range"
            min="0"
            max="1"
            step="0.05"
            bind:value={draft.overlay.backgroundOpacity}
          />
        </label>

        <label>
          <span>Font size</span>
          <input
            type="number"
            min="16"
            max="72"
            step="1"
            bind:value={draft.overlay.fontSize}
          />
        </label>

        <label>
          <span>Font family</span>
          <input type="text" bind:value={draft.overlay.fontFamily} />
        </label>

        <label>
          <span>Text color</span>
          <input type="color" bind:value={draft.overlay.textColor} />
        </label>

        <label class="toggle-row">
          <span>Original + translation</span>
          <input
            type="checkbox"
            bind:checked={draft.overlay.showOriginalAndTranslation}
          />
        </label>
      {/if}

      {#if errorMessage}
        <p class="error-message" role="alert">{errorMessage}</p>
      {/if}
    </div>

    <footer class="panel-footer">
      <button class="secondary-action" type="button" onclick={onCancel}>
        Cancel
      </button>
      <button
        class="secondary-action"
        type="button"
        disabled={panelState !== "idle"}
        onclick={testCaption}
      >
        {panelState === "testing" ? "Opening" : "Test caption"}
      </button>
      <button
        class="primary-action"
        type="button"
        disabled={panelState !== "idle"}
        onclick={saveSettings}
      >
        {panelState === "saving" ? "Saving" : "Save"}
      </button>
    </footer>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 10;
    display: grid;
    place-items: center;
    padding: 24px;
    background: oklch(0% 0 0 / 0.58);
  }

  .settings-panel {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr) auto;
    width: min(520px, 100%);
    max-height: min(700px, calc(100vh - 48px));
    overflow: hidden;
    border: 1px solid var(--borderColor-default);
    border-radius: var(--radius-default);
    background: var(--bgColor-raised);
    box-shadow: 0 24px 70px oklch(0% 0 0 / 0.38);
  }

  .panel-header,
  .panel-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 18px 20px;
  }

  .panel-header {
    border-bottom: 1px solid var(--borderColor-muted);
  }

  .panel-footer {
    flex-wrap: wrap;
    border-top: 1px solid var(--borderColor-muted);
  }

  h2,
  p {
    margin: 0;
  }

  h2 {
    font-size: 1.25rem;
    letter-spacing: 0;
  }

  .settings-body {
    display: grid;
    min-height: 0;
    gap: 14px;
    overflow-y: auto;
    padding: 20px;
    scrollbar-width: none;
  }

  .settings-body::-webkit-scrollbar {
    width: 0;
    height: 0;
  }

  label {
    display: grid;
    gap: 8px;
    color: var(--fgColor-muted);
    font-size: 0.875rem;
  }

  input {
    min-height: 38px;
    box-sizing: border-box;
    border: 1px solid var(--borderColor-default);
    border-radius: var(--radius-default);
    padding: 0 10px;
    background: var(--bgColor-muted);
    color: var(--fgColor-default);
  }

  input[type="range"],
  input[type="checkbox"],
  input[type="color"] {
    cursor: pointer;
  }

  .toggle-row {
    grid-template-columns: 1fr auto;
    align-items: center;
  }

  .toggle-row input {
    width: 18px;
    min-height: 18px;
    padding: 0;
  }

  .icon-button,
  .primary-action,
  .secondary-action {
    min-height: 38px;
    border: 1px solid transparent;
    border-radius: var(--radius-default);
    padding: 0 14px;
    cursor: pointer;
    font-weight: 700;
  }

  .icon-button {
    width: 36px;
    padding: 0;
    border-color: var(--borderColor-default);
    background: var(--bgColor-muted);
    color: var(--fgColor-muted);
  }

  .icon-button:hover,
  .secondary-action:hover {
    background: var(--button-secondary-bgColor-hover);
    color: var(--fgColor-default);
  }

  .primary-action {
    background: var(--button-primary-bgColor-rest);
    color: var(--fgColor-onEmphasis);
  }

  .primary-action:hover {
    background: var(--button-primary-bgColor-hover);
  }

  .secondary-action {
    border-color: var(--borderColor-default);
    background: transparent;
    color: var(--fgColor-default);
  }

  button:disabled {
    cursor: default;
    opacity: 0.52;
  }

  .error-message {
    color: oklch(76% 0.15 25);
    font-size: 0.875rem;
  }
</style>
