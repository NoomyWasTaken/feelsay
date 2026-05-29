<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  const currentWindow = getCurrentWindow();

  async function startDragging() {
    try {
      await currentWindow.startDragging();
    } catch (error) {
      console.error(error);
    }
  }

  async function minimizeWindow() {
    try {
      await currentWindow.minimize();
    } catch (error) {
      console.error(error);
    }
  }

  async function toggleMaximizeWindow() {
    try {
      await currentWindow.toggleMaximize();
    } catch (error) {
      console.error(error);
    }
  }

  async function closeWindow() {
    try {
      await currentWindow.close();
    } catch (error) {
      console.error(error);
    }
  }
</script>

<header class="titlebar">
  <button
    class="drag-region"
    type="button"
    tabindex="-1"
    aria-label="Move window"
    onpointerdown={startDragging}
  >
    <span class="app-dot" aria-hidden="true"></span>
    <span>FeelSay</span>
  </button>

  <div class="window-controls" aria-label="Window controls">
    <button type="button" aria-label="Minimize" onclick={minimizeWindow}>
      <span class="control-icon minimize" aria-hidden="true"></span>
    </button>
    <button type="button" aria-label="Maximize" onclick={toggleMaximizeWindow}>
      <span class="control-icon maximize" aria-hidden="true"></span>
    </button>
    <button
      class="close"
      type="button"
      aria-label="Close"
      onclick={closeWindow}
    >
      <span class="control-icon close-icon" aria-hidden="true"></span>
    </button>
  </div>
</header>

<style>
  .titlebar {
    display: grid;
    grid-template-columns: 1fr auto;
    height: 34px;
    background: oklch(12% 0.006 240);
    color: var(--fgColor-default);
    user-select: none;
  }

  .drag-region {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    border: 0;
    padding: 0 10px;
    background: transparent;
    color: var(--fgColor-muted);
    cursor: default;
    font-size: 0.8125rem;
    font-weight: 650;
    text-align: left;
  }

  .app-dot {
    width: 10px;
    height: 10px;
    flex: 0 0 auto;
    border-radius: 50%;
    background: var(--button-primary-bgColor-rest);
    box-shadow: inset 0 0 0 3px oklch(12% 0.006 240);
  }

  .window-controls {
    display: flex;
    height: 100%;
  }

  .window-controls button {
    display: grid;
    width: 46px;
    height: 34px;
    place-items: center;
    border: 0;
    background: transparent;
    color: var(--fgColor-muted);
    cursor: default;
  }

  .window-controls button:hover {
    background: oklch(100% 0 0 / 0.08);
    color: var(--fgColor-default);
  }

  .window-controls button.close:hover {
    background: oklch(58% 0.2 25);
    color: white;
  }

  .control-icon {
    position: relative;
    display: block;
    width: 12px;
    height: 12px;
  }

  .minimize::before {
    position: absolute;
    top: 6px;
    left: 1px;
    width: 10px;
    height: 1px;
    background: currentColor;
    content: "";
  }

  .maximize {
    box-sizing: border-box;
    border: 1px solid currentColor;
  }

  .close-icon::before,
  .close-icon::after {
    position: absolute;
    top: 5px;
    left: 0;
    width: 12px;
    height: 1px;
    background: currentColor;
    content: "";
  }

  .close-icon::before {
    transform: rotate(45deg);
  }

  .close-icon::after {
    transform: rotate(-45deg);
  }
</style>
