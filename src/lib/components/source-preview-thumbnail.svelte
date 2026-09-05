<script lang="ts">
  import type { SourcePreview } from "$lib/domain/source-selection";

  type Props = {
    fallbackInitials: string;
    preview: SourcePreview;
  };

  let { fallbackInitials, preview }: Props = $props();
</script>

<span class="preview-shell" aria-label={`${preview.title} preview`}>
  {#if preview.thumbnailUrl || preview.thumbnailData}
    <img
      alt=""
      src={preview.thumbnailUrl ?? preview.thumbnailData}
      draggable="false"
    />
  {:else if preview.mockTemplate === "browser"}
    <span class="mock browser">
      <span class="browser-bar">
        <span></span>
        <span></span>
        <span></span>
      </span>
      <span class="browser-body">
        <span class="browser-sidebar"></span>
        <span class="browser-content">
          <span></span>
          <span></span>
          <span></span>
        </span>
      </span>
    </span>
  {:else if preview.mockTemplate === "chat"}
    <span class="mock chat">
      <span class="chat-rail"></span>
      <span class="chat-list">
        <span></span>
        <span></span>
        <span></span>
      </span>
      <span class="chat-thread">
        <span></span>
        <span></span>
        <span></span>
      </span>
    </span>
  {:else if preview.mockTemplate === "meeting"}
    <span class="mock meeting">
      <span></span>
      <span></span>
      <span></span>
      <span></span>
    </span>
  {:else if preview.mockTemplate === "video"}
    <span class="mock video">
      <span class="play"></span>
      <span class="timeline"></span>
    </span>
  {:else if preview.mockTemplate === "game"}
    <span class="mock game">
      <span class="game-horizon"></span>
      <span class="game-panel left"></span>
      <span class="game-panel right"></span>
      <span class="game-map"></span>
    </span>
  {:else if preview.cssPreview}
    <span
      class="mock css-preview"
      style:--preview-accent={preview.cssPreview.accentColor}
      style:--preview-background={preview.cssPreview.background}
    >
      <span class="css-preview-bar"></span>
      <span class="css-preview-body">
        <span></span>
        <span></span>
        <span></span>
      </span>
    </span>
  {:else}
    <span class="mock fallback"></span>
  {/if}

  <span class="fallback-icon" aria-hidden="true">{fallbackInitials}</span>
</span>

<style>
  .preview-shell {
    position: relative;
    display: block;
    aspect-ratio: 16 / 10;
    overflow: hidden;
    border: 1px solid oklch(100% 0 0 / 0.08);
    border-radius: 7px;
    background: oklch(12% 0.008 240);
  }

  img,
  .mock {
    display: block;
    width: 100%;
    height: 100%;
  }

  img {
    object-fit: cover;
  }

  .fallback-icon {
    position: absolute;
    right: 8px;
    bottom: 8px;
    display: grid;
    min-width: 28px;
    height: 24px;
    place-items: center;
    border: 1px solid oklch(100% 0 0 / 0.14);
    border-radius: 6px;
    padding: 0 6px;
    background: oklch(10% 0.008 240 / 0.72);
    color: oklch(92% 0.006 240);
    font-size: 0.68rem;
    font-weight: 800;
  }

  .mock {
    position: relative;
  }

  .css-preview {
    display: grid;
    grid-template-rows: 20px 1fr;
    background: var(--preview-background);
  }

  .css-preview-bar {
    background: oklch(100% 0 0 / 0.1);
  }

  .css-preview-body {
    display: grid;
    align-content: center;
    gap: 8px;
    padding: 16px 18px;
  }

  .css-preview-body span {
    display: block;
    height: 8px;
    border-radius: 999px;
    background: var(--preview-accent);
    opacity: 0.74;
  }

  .css-preview-body span:nth-child(2) {
    width: 72%;
    opacity: 0.42;
  }

  .css-preview-body span:nth-child(3) {
    width: 48%;
    opacity: 0.26;
  }

  .browser {
    background: linear-gradient(
      135deg,
      oklch(23% 0.04 235),
      oklch(16% 0.02 245)
    );
  }

  .browser-bar {
    display: flex;
    gap: 4px;
    padding: 8px;
    background: oklch(100% 0 0 / 0.08);
  }

  .browser-bar span {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: oklch(74% 0.12 195);
  }

  .browser-body {
    display: grid;
    grid-template-columns: 32% 1fr;
    gap: 8px;
    padding: 10px;
  }

  .browser-sidebar,
  .browser-content span,
  .chat-list span,
  .chat-thread span {
    display: block;
    border-radius: 999px;
    background: oklch(100% 0 0 / 0.16);
  }

  .browser-sidebar {
    height: 54px;
    border-radius: 6px;
  }

  .browser-content {
    display: grid;
    align-content: start;
    gap: 7px;
  }

  .browser-content span {
    height: 8px;
  }

  .browser-content span:nth-child(2) {
    width: 78%;
  }

  .browser-content span:nth-child(3) {
    width: 56%;
  }

  .chat {
    display: grid;
    grid-template-columns: 22px 42px 1fr;
    gap: 8px;
    padding: 10px;
    box-sizing: border-box;
    background: linear-gradient(
      135deg,
      oklch(24% 0.08 282),
      oklch(16% 0.025 250)
    );
  }

  .chat-rail {
    border-radius: 8px;
    background: oklch(100% 0 0 / 0.14);
  }

  .chat-list,
  .chat-thread {
    display: grid;
    align-content: start;
    gap: 8px;
  }

  .chat-list span {
    height: 18px;
    border-radius: 6px;
  }

  .chat-thread span {
    height: 10px;
  }

  .chat-thread span:nth-child(2) {
    width: 72%;
  }

  .meeting {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
    padding: 10px;
    box-sizing: border-box;
    background: linear-gradient(
      135deg,
      oklch(22% 0.055 235),
      oklch(15% 0.018 245)
    );
  }

  .meeting span {
    border-radius: 8px;
    background:
      radial-gradient(
        circle at 50% 34%,
        oklch(76% 0.09 195) 0 12%,
        transparent 13%
      ),
      radial-gradient(
        circle at 50% 70%,
        oklch(100% 0 0 / 0.18) 0 20%,
        transparent 21%
      ),
      oklch(100% 0 0 / 0.09);
  }

  .video {
    background:
      linear-gradient(transparent 76%, oklch(0% 0 0 / 0.38) 77%),
      radial-gradient(
        circle at 34% 36%,
        oklch(82% 0.16 55 / 0.72),
        transparent 26%
      ),
      linear-gradient(135deg, oklch(19% 0.03 35), oklch(10% 0.012 245));
  }

  .play {
    position: absolute;
    top: 39%;
    left: 47%;
    width: 0;
    height: 0;
    border-top: 10px solid transparent;
    border-bottom: 10px solid transparent;
    border-left: 15px solid oklch(96% 0 0 / 0.86);
  }

  .timeline {
    position: absolute;
    right: 12px;
    bottom: 12px;
    left: 12px;
    height: 4px;
    border-radius: 999px;
    background: linear-gradient(
      90deg,
      oklch(72% 0.17 35) 48%,
      oklch(100% 0 0 / 0.22) 49%
    );
  }

  .game {
    background:
      radial-gradient(
        circle at 72% 30%,
        oklch(72% 0.14 95 / 0.42),
        transparent 22%
      ),
      linear-gradient(180deg, oklch(20% 0.05 250), oklch(13% 0.045 150));
  }

  .game-horizon {
    position: absolute;
    right: 0;
    bottom: 28px;
    left: 0;
    height: 28px;
    background: oklch(31% 0.09 150 / 0.78);
    clip-path: polygon(
      0 58%,
      25% 30%,
      48% 62%,
      68% 20%,
      100% 54%,
      100% 100%,
      0 100%
    );
  }

  .game-panel,
  .game-map {
    position: absolute;
    border: 1px solid oklch(100% 0 0 / 0.12);
    background: oklch(0% 0 0 / 0.26);
  }

  .game-panel {
    bottom: 10px;
    width: 34px;
    height: 18px;
    border-radius: 5px;
  }

  .game-panel.left {
    left: 10px;
  }

  .game-panel.right {
    right: 10px;
  }

  .game-map {
    top: 10px;
    right: 10px;
    width: 28px;
    height: 28px;
    border-radius: 50%;
  }

  .fallback {
    background: oklch(18% 0.016 245);
  }
</style>
