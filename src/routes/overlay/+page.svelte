<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { PhysicalPosition } from "@tauri-apps/api/dpi";
  import { cursorPosition, getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import type { AppSettings } from "$lib/domain/settings";
  import {
    commandErrorMessage,
    destroyOverlay,
    getSettings,
  } from "$lib/tauri/commands";

  type OverlayState =
    | { status: "loading" }
    | { status: "ready"; settings: AppSettings }
    | { status: "error"; message: string };

  let overlayState = $state<OverlayState>({ status: "loading" });
  let isManualDragging = false;

  onMount(() => {
    document.documentElement.classList.add("overlay-window");
    let unlistenSettings: (() => void) | undefined;

    void loadSettings();
    void listen<AppSettings>("settings-updated", (event) => {
      overlayState = {
        status: "ready",
        settings: event.payload,
      };
    }).then((unlisten) => {
      unlistenSettings = unlisten;
    });

    return () => {
      unlistenSettings?.();
      document.documentElement.classList.remove("overlay-window");
    };
  });

  async function loadSettings() {
    try {
      overlayState = {
        status: "ready",
        settings: await getSettings(),
      };
    } catch (error) {
      overlayState = {
        status: "error",
        message: commandErrorMessage(error),
      };
    }
  }

  async function startDragging(event: PointerEvent) {
    if (event.button !== 0) {
      return;
    }

    if (event.target instanceof Element) {
      if (event.target.closest("[data-overlay-control]")) {
        return;
      }
    }

    await startManualDragging(event);
  }

  async function startManualDragging(event: PointerEvent) {
    if (isManualDragging) {
      return;
    }

    event.preventDefault();
    const dragTarget =
      event.currentTarget instanceof HTMLElement ? event.currentTarget : null;
    dragTarget?.setPointerCapture(event.pointerId);

    const appWindow = getCurrentWindow();
    const originWindow = await appWindow.outerPosition();
    const originCursor = await cursorPosition();
    isManualDragging = true;

    async function moveWindow() {
      if (!isManualDragging) {
        return;
      }

      const cursor = await cursorPosition();
      await appWindow.setPosition(
        new PhysicalPosition(
          originWindow.x + cursor.x - originCursor.x,
          originWindow.y + cursor.y - originCursor.y,
        ),
      );
    }

    function stopDragging() {
      isManualDragging = false;
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", stopDragging);
      window.removeEventListener("pointercancel", stopDragging);

      try {
        dragTarget?.releasePointerCapture(event.pointerId);
      } catch {
        // Pointer capture may already be released by the WebView.
      }
    }

    function handlePointerMove() {
      void moveWindow().catch((error) => {
        console.error(commandErrorMessage(error));
        stopDragging();
      });
    }

    window.addEventListener("pointermove", handlePointerMove);
    window.addEventListener("pointerup", stopDragging, { once: true });
    window.addEventListener("pointercancel", stopDragging, { once: true });
  }

  function stopOverlayControlPointer(event: PointerEvent) {
    event.stopPropagation();
  }

  async function closeOverlay(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    console.log("overlay close button clicked");

    const fallbackTimer = window.setTimeout(() => {
      void destroyOverlay().catch((error) => {
        console.error(commandErrorMessage(error));
      });
    }, 250);

    try {
      await getCurrentWindow().close();
    } catch (error) {
      window.clearTimeout(fallbackTimer);
      console.error(commandErrorMessage(error));
      await destroyOverlay();
    }
  }
</script>

<svelte:head>
  <title>FeelSay Overlay</title>
</svelte:head>

{#if overlayState.status === "ready"}
  <main
    class="overlay-shell"
    aria-live="polite"
    style:opacity={overlayState.settings.overlay.opacity}
    style:font-family={overlayState.settings.overlay.fontFamily}
    style:font-size={`${overlayState.settings.overlay.fontSize}px`}
    style:color={overlayState.settings.overlay.textColor}
    style:--caption-background-opacity={overlayState.settings.overlay
      .backgroundOpacity}
    onpointerdown={startDragging}
  >
    <div class="overlay-header">
      <span>FeelSay</span>
      <button
        class="close-button"
        type="button"
        aria-label="Close overlay"
        title="Close"
        data-overlay-control
        onpointerdown={stopOverlayControlPointer}
        onclick={closeOverlay}
      >
        x
      </button>
    </div>
    <p class="caption-text">Live captions will appear here.</p>
  </main>
{:else if overlayState.status === "error"}
  <main
    class="overlay-shell fallback"
    aria-live="polite"
    onpointerdown={startDragging}
  >
    <p class="caption-text">{overlayState.message}</p>
  </main>
{:else}
  <main
    class="overlay-shell fallback"
    aria-live="polite"
    onpointerdown={startDragging}
  >
    <p class="caption-text">Loading caption overlay</p>
  </main>
{/if}

<style>
  :global(html),
  :global(body) {
    width: 100%;
    height: 100%;
    background: transparent !important;
    overflow: hidden;
  }

  .overlay-shell {
    display: grid;
    grid-template-rows: auto 1fr;
    width: 100vw;
    height: 100vh;
    box-sizing: border-box;
    overflow: hidden;
    padding: 0 18px 24px;
    border: 1px solid oklch(100% 0 0 / 0.18);
    border-radius: 8px;
    background: oklch(16% 0.018 245 / var(--caption-background-opacity));
    backdrop-filter: blur(14px);
    cursor: grab;
    user-select: none;
  }

  .overlay-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-width: 0;
    height: 30px;
    border: 0;
    border-bottom: 1px solid oklch(100% 0 0 / 0.08);
    padding: 0;
    background: transparent;
    color: oklch(86% 0.008 245);
    text-align: left;
    font:
      700 12px/1 Inter,
      "Segoe UI",
      system-ui,
      sans-serif;
  }

  .overlay-shell:active {
    cursor: grabbing;
  }

  .overlay-header span {
    pointer-events: none;
  }

  .close-button {
    display: grid;
    width: 26px;
    height: 26px;
    place-items: center;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: oklch(86% 0.008 245);
    cursor: pointer;
    font-size: 18px;
    line-height: 1;
  }

  .close-button:hover {
    background: oklch(100% 0 0 / 0.1);
    color: oklch(98% 0 0);
  }

  .caption-text {
    align-self: center;
    justify-self: center;
    max-width: 26ch;
    margin: 0;
    text-align: center;
    text-shadow: 0 2px 12px oklch(0% 0 0 / 0.32);
  }

  .fallback {
    grid-template-rows: 1fr;
    padding: 24px;
    color: #f8fafc;
    font:
      600 24px/1.35 Inter,
      "Segoe UI",
      system-ui,
      sans-serif;
    --caption-background-opacity: 0.72;
  }
</style>
