<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import {
    translationLanguageOptions,
    type CaptionSettings,
    type TranslationLanguage,
  } from "$lib/domain/caption-settings";
  import {
    formatModelSize,
    type ModelSettingsStore,
  } from "$lib/domain/model-settings";
  import type {
    TranscriptExportFormat,
    TranscriptSessionSummary,
    TranscriptSettings,
  } from "$lib/domain/transcript-settings";
  import type { PerformanceSettings } from "$lib/domain/performance-settings";
  import type {
    TranslationEngineStatus,
    TranslationSettings,
  } from "$lib/domain/translation-settings";
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
    type AppMetadata,
    commandErrorMessage,
    exportTranscriptSession,
    getAppMetadata,
    getCaptionSettings,
    getModelSettingsStore,
    getOverlayPlacement,
    getOverlaySettingsStore,
    getPerformanceSettings,
    getTranscriptSettings,
    getTranslationEngineStatus,
    getTranslationSettings,
    installDefaultAsrAssets,
    listTranscriptSessions,
    runAsrModelDiagnostic,
    runCaptionFlowDiagnostic,
    runTranslationEngineDiagnostic,
    saveCaptionSettings,
    saveModelSettingsStore,
    saveOverlaySettingsStore,
    savePerformanceSettings,
    saveTranscriptSettings,
    saveTranslationSettings,
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
  let appMetadata = $state<AppMetadata | null>(null);
  let settingsStore = $state<OverlaySettingsStore | null>(null);
  let captionSettings = $state<CaptionSettings | null>(null);
  let modelStore = $state<ModelSettingsStore | null>(null);
  let performanceSettings = $state<PerformanceSettings | null>(null);
  let translationSettings = $state<TranslationSettings | null>(null);
  let translationStatus = $state<TranslationEngineStatus | null>(null);
  let transcriptSettings = $state<TranscriptSettings | null>(null);
  let transcriptSessions = $state<TranscriptSessionSummary[]>([]);
  let saveState = $state<SaveState>({ status: "saved" });
  let captionSaveState = $state<SaveState>({ status: "saved" });
  let modelSaveState = $state<SaveState>({ status: "saved" });
  let performanceSaveState = $state<SaveState>({ status: "saved" });
  let translationSaveState = $state<SaveState>({ status: "saved" });
  let modelInstallState = $state<SaveState>({ status: "saved" });
  let modelDiagnosticState = $state<
    | { status: "idle" }
    | { status: "running" }
    | { status: "done"; text: string }
    | { status: "error"; message: string }
  >({ status: "idle" });
  let translationDiagnosticState = $state<
    | { status: "idle" }
    | { status: "running" }
    | { status: "done"; text: string }
    | { status: "error"; message: string }
  >({ status: "idle" });
  let transcriptSaveState = $state<SaveState>({ status: "saved" });
  let placement = $state<OverlayPlacement | null>(null);
  let placementOrigin = $state<OverlayPlacement | null>(null);
  let placementState = $state<"idle" | "starting" | "active" | "error">("idle");
  let placementMessage = $state("");
  let transcriptHistoryState = $state<
    | { status: "idle" }
    | { status: "loading" }
    | { status: "exported"; path: string }
    | { status: "error"; message: string }
  >({ status: "idle" });
  let placementPoll: ReturnType<typeof setInterval> | undefined;
  let autosaveTimer: ReturnType<typeof setTimeout> | undefined;
  let captionAutosaveTimer: ReturnType<typeof setTimeout> | undefined;
  let modelAutosaveTimer: ReturnType<typeof setTimeout> | undefined;
  let performanceAutosaveTimer: ReturnType<typeof setTimeout> | undefined;
  let translationAutosaveTimer: ReturnType<typeof setTimeout> | undefined;
  let transcriptAutosaveTimer: ReturnType<typeof setTimeout> | undefined;
  let lastSavedStoreJson = "";
  let lastSavedCaptionSettingsJson = "";
  let lastSavedModelStoreJson = "";
  let lastSavedPerformanceSettingsJson = "";
  let lastSavedTranslationSettingsJson = "";
  let lastSavedTranscriptSettingsJson = "";
  let saveRequestId = 0;
  let captionSaveRequestId = 0;
  let modelSaveRequestId = 0;
  let performanceSaveRequestId = 0;
  let translationSaveRequestId = 0;
  let transcriptSaveRequestId = 0;

  onMount(() => {
    void loadSettings();
  });

  onDestroy(() => {
    stopAutosaveTimer();
    stopCaptionAutosaveTimer();
    stopModelAutosaveTimer();
    stopPerformanceAutosaveTimer();
    stopTranslationAutosaveTimer();
    stopTranscriptAutosaveTimer();
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

  $effect(() => {
    const store = modelStore;

    if (!store || settingsState.status !== "ready") {
      return;
    }

    const serializedStore = JSON.stringify(store);

    if (serializedStore === lastSavedModelStoreJson) {
      return;
    }

    modelSaveState = { status: "saving" };
    stopModelAutosaveTimer();

    const requestId = ++modelSaveRequestId;
    modelAutosaveTimer = setTimeout(() => {
      void autosaveModelSettings(store, requestId);
    }, 200);
  });

  $effect(() => {
    const settings = captionSettings;

    if (!settings || settingsState.status !== "ready") {
      return;
    }

    const serializedSettings = JSON.stringify(settings);

    if (serializedSettings === lastSavedCaptionSettingsJson) {
      return;
    }

    captionSaveState = { status: "saving" };
    stopCaptionAutosaveTimer();

    const requestId = ++captionSaveRequestId;
    captionAutosaveTimer = setTimeout(() => {
      void autosaveCaptionSettings(settings, requestId);
    }, 200);
  });

  $effect(() => {
    const settings = performanceSettings;

    if (!settings || settingsState.status !== "ready") {
      return;
    }

    const serializedSettings = JSON.stringify(settings);

    if (serializedSettings === lastSavedPerformanceSettingsJson) {
      return;
    }

    performanceSaveState = { status: "saving" };
    stopPerformanceAutosaveTimer();

    const requestId = ++performanceSaveRequestId;
    performanceAutosaveTimer = setTimeout(() => {
      void autosavePerformanceSettings(settings, requestId);
    }, 200);
  });

  $effect(() => {
    const settings = translationSettings;

    if (!settings || settingsState.status !== "ready") {
      return;
    }

    const serializedSettings = JSON.stringify(settings);

    if (serializedSettings === lastSavedTranslationSettingsJson) {
      return;
    }

    translationSaveState = { status: "saving" };
    stopTranslationAutosaveTimer();

    const requestId = ++translationSaveRequestId;
    translationAutosaveTimer = setTimeout(() => {
      void autosaveTranslationSettings(settings, requestId);
    }, 200);
  });

  $effect(() => {
    const settings = transcriptSettings;

    if (!settings || settingsState.status !== "ready") {
      return;
    }

    const serializedSettings = JSON.stringify(settings);

    if (serializedSettings === lastSavedTranscriptSettingsJson) {
      return;
    }

    transcriptSaveState = { status: "saving" };
    stopTranscriptAutosaveTimer();

    const requestId = ++transcriptSaveRequestId;
    transcriptAutosaveTimer = setTimeout(() => {
      void autosaveTranscriptSettings(settings, requestId);
    }, 200);
  });

  async function loadSettings() {
    try {
      const [
        loadedSettingsStore,
        loadedAppMetadata,
        loadedCaptionSettings,
        loadedModelStore,
        loadedPerformanceSettings,
        loadedTranslationSettings,
        loadedTranslationStatus,
        loadedTranscriptSettings,
        loadedTranscriptSessions,
      ] = await Promise.all([
        getOverlaySettingsStore(),
        getAppMetadata(),
        getCaptionSettings(),
        getModelSettingsStore(),
        getPerformanceSettings(),
        getTranslationSettings(),
        getTranslationEngineStatus(),
        getTranscriptSettings(),
        listTranscriptSessions(),
      ]);
      settingsStore = loadedSettingsStore;
      appMetadata = loadedAppMetadata;
      captionSettings = loadedCaptionSettings;
      modelStore = loadedModelStore;
      performanceSettings = loadedPerformanceSettings;
      translationSettings = loadedTranslationSettings;
      translationStatus = loadedTranslationStatus;
      transcriptSettings = loadedTranscriptSettings;
      transcriptSessions = loadedTranscriptSessions;
      lastSavedStoreJson = serializeStore(settingsStore);
      lastSavedCaptionSettingsJson = JSON.stringify(captionSettings);
      lastSavedModelStoreJson = JSON.stringify(modelStore);
      lastSavedPerformanceSettingsJson = JSON.stringify(performanceSettings);
      lastSavedTranslationSettingsJson = JSON.stringify(translationSettings);
      lastSavedTranscriptSettingsJson = JSON.stringify(transcriptSettings);
      saveState = { status: "saved" };
      captionSaveState = { status: "saved" };
      modelSaveState = { status: "saved" };
      performanceSaveState = { status: "saved" };
      translationSaveState = { status: "saved" };
      transcriptSaveState = { status: "saved" };
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

  function stopModelAutosaveTimer() {
    if (modelAutosaveTimer) {
      clearTimeout(modelAutosaveTimer);
      modelAutosaveTimer = undefined;
    }
  }

  function stopCaptionAutosaveTimer() {
    if (captionAutosaveTimer) {
      clearTimeout(captionAutosaveTimer);
      captionAutosaveTimer = undefined;
    }
  }

  function stopPerformanceAutosaveTimer() {
    if (performanceAutosaveTimer) {
      clearTimeout(performanceAutosaveTimer);
      performanceAutosaveTimer = undefined;
    }
  }

  function stopTranslationAutosaveTimer() {
    if (translationAutosaveTimer) {
      clearTimeout(translationAutosaveTimer);
      translationAutosaveTimer = undefined;
    }
  }

  function stopTranscriptAutosaveTimer() {
    if (transcriptAutosaveTimer) {
      clearTimeout(transcriptAutosaveTimer);
      transcriptAutosaveTimer = undefined;
    }
  }

  async function autosaveCaptionSettings(
    settingsSnapshot: CaptionSettings,
    requestId: number,
  ) {
    try {
      const savedSettings = await saveCaptionSettings(settingsSnapshot);

      if (requestId !== captionSaveRequestId) {
        return;
      }

      lastSavedCaptionSettingsJson = JSON.stringify(savedSettings);
      captionSettings = savedSettings;
      captionSaveState = { status: "saved" };
    } catch (error) {
      if (requestId !== captionSaveRequestId) {
        return;
      }

      captionSaveState = {
        status: "error",
        message: commandErrorMessage(error),
      };
    }
  }

  async function autosaveModelSettings(
    storeSnapshot: ModelSettingsStore,
    requestId: number,
  ) {
    try {
      const savedStore = await saveModelSettingsStore(storeSnapshot);

      if (requestId !== modelSaveRequestId) {
        return;
      }

      lastSavedModelStoreJson = JSON.stringify(savedStore);
      modelStore = savedStore;
      modelSaveState = { status: "saved" };
    } catch (error) {
      if (requestId !== modelSaveRequestId) {
        return;
      }

      modelSaveState = {
        status: "error",
        message: commandErrorMessage(error),
      };
    }
  }

  async function autosavePerformanceSettings(
    settingsSnapshot: PerformanceSettings,
    requestId: number,
  ) {
    try {
      const savedSettings = await savePerformanceSettings(settingsSnapshot);

      if (requestId !== performanceSaveRequestId) {
        return;
      }

      lastSavedPerformanceSettingsJson = JSON.stringify(savedSettings);
      performanceSettings = savedSettings;
      performanceSaveState = { status: "saved" };
    } catch (error) {
      if (requestId !== performanceSaveRequestId) {
        return;
      }

      performanceSaveState = {
        status: "error",
        message: commandErrorMessage(error),
      };
    }
  }

  async function autosaveTranslationSettings(
    settingsSnapshot: TranslationSettings,
    requestId: number,
  ) {
    try {
      const savedSettings = await saveTranslationSettings(settingsSnapshot);

      if (requestId !== translationSaveRequestId) {
        return;
      }

      lastSavedTranslationSettingsJson = JSON.stringify(savedSettings);
      translationSettings = savedSettings;
      translationStatus = await getTranslationEngineStatus();
      translationSaveState = { status: "saved" };
    } catch (error) {
      if (requestId !== translationSaveRequestId) {
        return;
      }

      translationSaveState = {
        status: "error",
        message: commandErrorMessage(error),
      };
    }
  }

  async function autosaveTranscriptSettings(
    settingsSnapshot: TranscriptSettings,
    requestId: number,
  ) {
    try {
      const savedSettings = await saveTranscriptSettings(settingsSnapshot);

      if (requestId !== transcriptSaveRequestId) {
        return;
      }

      lastSavedTranscriptSettingsJson = JSON.stringify(savedSettings);
      transcriptSettings = savedSettings;
      transcriptSaveState = { status: "saved" };
    } catch (error) {
      if (requestId !== transcriptSaveRequestId) {
        return;
      }

      transcriptSaveState = {
        status: "error",
        message: commandErrorMessage(error),
      };
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

  function getActiveModel(store: ModelSettingsStore) {
    return (
      store.models.find((model) => model.id === store.activeModelId) ??
      store.models[0]
    );
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

  function selectModel(modelId: string) {
    if (!modelStore) {
      return;
    }

    modelStore = {
      ...modelStore,
      activeModelId: modelId,
    };
  }

  function updateActiveModelPath(path: string) {
    const store = modelStore;

    if (!store) {
      return;
    }

    modelStore = {
      ...store,
      models: store.models.map((model) =>
        model.id === store.activeModelId ? { ...model, path } : model,
      ),
    };
  }

  function updateActiveModelExecutablePath(executablePath: string) {
    const store = modelStore;

    if (!store) {
      return;
    }

    modelStore = {
      ...store,
      models: store.models.map((model) =>
        model.id === store.activeModelId ? { ...model, executablePath } : model,
      ),
    };
  }

  async function installDefaultModel() {
    modelInstallState = { status: "saving" };

    try {
      const installedStore = await installDefaultAsrAssets();
      modelStore = installedStore;
      lastSavedModelStoreJson = JSON.stringify(installedStore);
      modelInstallState = { status: "saved" };
    } catch (error) {
      modelInstallState = {
        status: "error",
        message: commandErrorMessage(error),
      };
    }
  }

  async function testAsrModel() {
    modelDiagnosticState = { status: "running" };

    try {
      const result = await runAsrModelDiagnostic();
      modelDiagnosticState = { status: "done", text: result.text };
    } catch (error) {
      modelDiagnosticState = {
        status: "error",
        message: commandErrorMessage(error),
      };
    }
  }

  async function testCaptionFlow() {
    modelDiagnosticState = { status: "running" };

    try {
      const result = await runCaptionFlowDiagnostic();
      modelDiagnosticState = { status: "done", text: result.text };
    } catch (error) {
      modelDiagnosticState = {
        status: "error",
        message: commandErrorMessage(error),
      };
    }
  }

  function updateTranscriptSaving(savingEnabled: boolean) {
    const settings = transcriptSettings;

    if (!settings) {
      return;
    }

    transcriptSettings = {
      ...settings,
      savingEnabled,
    };
  }

  function updateSpeakerLabels(speakerLabelsEnabled: boolean) {
    updateCaptionSettings({ speakerLabelsEnabled });
  }

  function updateCaptionSettings(settingsPatch: Partial<CaptionSettings>) {
    const settings = captionSettings;

    if (!settings) {
      return;
    }

    captionSettings = {
      ...settings,
      ...settingsPatch,
    };
  }

  function updateTranslationSettings(
    settingsPatch: Partial<TranslationSettings>,
  ) {
    const settings = translationSettings;

    if (!settings) {
      return;
    }

    translationSettings = {
      ...settings,
      ...settingsPatch,
    };
  }

  async function testTranslationEngine() {
    translationDiagnosticState = { status: "running" };

    try {
      const result = await runTranslationEngineDiagnostic();
      translationDiagnosticState = {
        status: "done",
        text: result.translatedText,
      };
      translationStatus = await getTranslationEngineStatus();
    } catch (error) {
      translationDiagnosticState = {
        status: "error",
        message: commandErrorMessage(error),
      };
    }
  }

  function updatePerformanceSettings(
    settingsPatch: Partial<PerformanceSettings>,
  ) {
    const settings = performanceSettings;

    if (!settings) {
      return;
    }

    performanceSettings = {
      ...settings,
      ...settingsPatch,
    };
  }

  async function refreshTranscriptSessions() {
    transcriptHistoryState = { status: "loading" };

    try {
      transcriptSessions = await listTranscriptSessions();
      transcriptHistoryState = { status: "idle" };
    } catch (error) {
      transcriptHistoryState = {
        status: "error",
        message: commandErrorMessage(error),
      };
    }
  }

  async function exportTranscript(
    session: TranscriptSessionSummary,
    format: TranscriptExportFormat,
  ) {
    transcriptHistoryState = { status: "loading" };

    try {
      const path = await exportTranscriptSession(session.id, format);
      transcriptHistoryState = { status: "exported", path };
    } catch (error) {
      transcriptHistoryState = {
        status: "error",
        message: commandErrorMessage(error),
      };
    }
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
    stopAutosaveTimer();
    stopModelAutosaveTimer();
    stopPerformanceAutosaveTimer();
    stopTranslationAutosaveTimer();
    stopTranscriptAutosaveTimer();
    await stopPlacement();
    onClose();
  }

  function previewSurfaceStyle(settings: OverlaySettings): string {
    return `background: ${rgbaPreviewColor(settings.backgroundColor, settings.backgroundOpacity)}`;
  }

  function previewTextStyle(
    settings: OverlaySettings,
    fontSize = settings.fontSize,
  ): string {
    const textColor = normalizeCssColor(settings.textColor, "#ffffff");
    const outlineColor = normalizeCssColor(settings.outlineColor, "#000000");
    const fontWeight = settings.fontWeight === "bold" ? 700 : 400;

    return [
      `font-family: ${JSON.stringify(settings.fontFamily)}, sans-serif`,
      `font-size: ${fontSize}px`,
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

  function formatSessionTime(milliseconds: number): string {
    return new Date(milliseconds).toLocaleString();
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
      {#if appMetadata}
        <p class="app-version">
          {appMetadata.productName}
          {appMetadata.version}
        </p>
      {/if}
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
          {#if modelStore}
            {@const activeModel = getActiveModel(modelStore)}
            <fieldset>
              <legend>Model</legend>
              <label>
                ASR model
                <select
                  value={modelStore.activeModelId}
                  onchange={(event) => selectModel(textInput(event))}
                >
                  {#each modelStore.models as model}
                    <option value={model.id}>{model.name}</option>
                  {/each}
                </select>
              </label>
              <p
                class:model-ready={activeModel?.isInstalled}
                class="model-status"
              >
                {activeModel?.isInstalled ? "Ready" : "Missing"}
                {activeModel
                  ? ` - ${formatModelSize(activeModel.fileSizeBytes)}`
                  : ""}
              </p>
              <button
                type="button"
                class="secondary-action"
                disabled={modelInstallState.status === "saving"}
                onclick={() => void installDefaultModel()}
              >
                {modelInstallState.status === "saving"
                  ? "Installing"
                  : "Install default"}
              </button>
              <button
                type="button"
                class="secondary-action"
                disabled={modelDiagnosticState.status === "running"}
                onclick={() => void testAsrModel()}
              >
                {modelDiagnosticState.status === "running"
                  ? "Testing"
                  : "Test model"}
              </button>
              <button
                type="button"
                class="secondary-action"
                disabled={modelDiagnosticState.status === "running"}
                onclick={() => void testCaptionFlow()}
              >
                {modelDiagnosticState.status === "running"
                  ? "Testing"
                  : "Test caption flow"}
              </button>
              <label>
                Model file
                <input
                  type="text"
                  value={activeModel?.path ?? ""}
                  oninput={(event) => updateActiveModelPath(textInput(event))}
                />
              </label>
              <label>
                Whisper executable
                <input
                  type="text"
                  value={activeModel?.executablePath ?? ""}
                  oninput={(event) =>
                    updateActiveModelExecutablePath(textInput(event))}
                />
              </label>
              <p class="model-hint">
                Set a GGML model and whisper.cpp executable for local captions.
              </p>
              {#if modelInstallState.status === "error"}
                <p class="model-hint error" role="alert">
                  {modelInstallState.message}
                </p>
              {/if}
              {#if modelDiagnosticState.status === "done"}
                <p class="model-hint">Heard: {modelDiagnosticState.text}</p>
              {:else if modelDiagnosticState.status === "error"}
                <p class="model-hint error" role="alert">
                  {modelDiagnosticState.message}
                </p>
              {/if}
              {#if modelSaveState.status === "error"}
                <p class="model-hint error" role="alert">
                  {modelSaveState.message}
                </p>
              {/if}
            </fieldset>
          {/if}

          {#if captionSettings && translationSettings}
            <fieldset>
              <legend>Translation</legend>
              <div class="two-column-fields">
                <label>
                  From
                  <select
                    value={captionSettings.translationSourceLanguage}
                    onchange={(event) =>
                      updateCaptionSettings({
                        translationSourceLanguage: textInput(
                          event,
                        ) as TranslationLanguage,
                      })}
                  >
                    {#each translationLanguageOptions as language}
                      <option value={language.value}>{language.label}</option>
                    {/each}
                  </select>
                </label>
                <label>
                  To
                  <select
                    value={captionSettings.translationTargetLanguage}
                    onchange={(event) =>
                      updateCaptionSettings({
                        translationTargetLanguage:
                          textInput(event) === "auto"
                            ? "english"
                            : (textInput(event) as TranslationLanguage),
                      })}
                  >
                    {#each translationLanguageOptions.filter((language) => language.value !== "auto") as language}
                      <option value={language.value}>{language.label}</option>
                    {/each}
                  </select>
                </label>
              </div>
              <label>
                Argos executable
                <input
                  type="text"
                  value={translationSettings.executablePath}
                  oninput={(event) =>
                    updateTranslationSettings({
                      executablePath: textInput(event),
                    })}
                />
              </label>
              <p
                class:model-ready={translationStatus?.isAvailable}
                class="model-status"
              >
                {translationStatus?.isAvailable ? "Ready" : "Missing"}
                {translationStatus ? ` - ${translationStatus.message}` : ""}
              </p>
              <button
                type="button"
                class="secondary-action"
                disabled={translationDiagnosticState.status === "running"}
                onclick={() => void testTranslationEngine()}
              >
                {translationDiagnosticState.status === "running"
                  ? "Testing"
                  : "Test translation"}
              </button>
              <p class="model-hint">
                Uses local Argos Translate packages for non-English targets.
              </p>
              {#if translationDiagnosticState.status === "done"}
                <p class="model-hint">
                  Translated: {translationDiagnosticState.text}
                </p>
              {:else if translationDiagnosticState.status === "error"}
                <p class="model-hint error" role="alert">
                  {translationDiagnosticState.message}
                </p>
              {/if}
              {#if captionSaveState.status !== "saved"}
                <p
                  class:error={captionSaveState.status === "error"}
                  class="model-hint"
                  role={captionSaveState.status === "error"
                    ? "alert"
                    : undefined}
                >
                  {saveStatusText(captionSaveState)}
                </p>
              {/if}
              {#if translationSaveState.status !== "saved"}
                <p
                  class:error={translationSaveState.status === "error"}
                  class="model-hint"
                  role={translationSaveState.status === "error"
                    ? "alert"
                    : undefined}
                >
                  {saveStatusText(translationSaveState)}
                </p>
              {/if}
            </fieldset>
          {/if}

          {#if performanceSettings}
            <fieldset>
              <legend>Performance</legend>
              <label>
                Preset
                <select
                  value={performanceSettings.preset}
                  onchange={(event) =>
                    updatePerformanceSettings({
                      preset: textInput(event) as PerformanceSettings["preset"],
                    })}
                >
                  <option value="low_resource">Low Resource</option>
                  <option value="balanced">Balanced</option>
                  <option value="accuracy">Accuracy</option>
                  <option value="custom">Custom</option>
                </select>
              </label>
              <p class="model-hint">{performanceSettings.resourceImpact}</p>

              {#if performanceSettings.preset === "custom"}
                <div class="two-column-fields">
                  <label>
                    Min chunk
                    <input
                      type="number"
                      min="1"
                      max="10"
                      step="0.5"
                      value={performanceSettings.minTranscribeSeconds}
                      oninput={(event) =>
                        updatePerformanceSettings({
                          minTranscribeSeconds: numberInput(event),
                        })}
                    />
                  </label>
                  <label>
                    Max chunk
                    <input
                      type="number"
                      min="2"
                      max="20"
                      step="0.5"
                      value={performanceSettings.maxTranscribeSeconds}
                      oninput={(event) =>
                        updatePerformanceSettings({
                          maxTranscribeSeconds: numberInput(event),
                        })}
                    />
                  </label>
                  <label>
                    Interval
                    <input
                      type="number"
                      min="500"
                      max="5000"
                      step="100"
                      value={performanceSettings.transcribeIntervalMs}
                      oninput={(event) =>
                        updatePerformanceSettings({
                          transcribeIntervalMs: numberInput(event),
                        })}
                    />
                  </label>
                  <label>
                    Queue
                    <input
                      type="number"
                      min="1"
                      max="8"
                      value={performanceSettings.asrQueueCapacity}
                      oninput={(event) =>
                        updatePerformanceSettings({
                          asrQueueCapacity: numberInput(event),
                        })}
                    />
                  </label>
                </div>
                <label>
                  Speech sensitivity
                  <span class="range-row">
                    <input
                      type="range"
                      min="0.01"
                      max="0.12"
                      step="0.005"
                      value={performanceSettings.vadSpeechLevelThreshold}
                      oninput={(event) =>
                        updatePerformanceSettings({
                          vadSpeechLevelThreshold: numberInput(event),
                        })}
                    />
                    <output>
                      {performanceSettings.vadSpeechLevelThreshold.toFixed(3)}
                    </output>
                  </span>
                </label>
                <label>
                  Silence release
                  <span class="range-row">
                    <input
                      type="range"
                      min="0.005"
                      max="0.08"
                      step="0.005"
                      value={performanceSettings.vadSilenceLevelThreshold}
                      oninput={(event) =>
                        updatePerformanceSettings({
                          vadSilenceLevelThreshold: numberInput(event),
                        })}
                    />
                    <output>
                      {performanceSettings.vadSilenceLevelThreshold.toFixed(3)}
                    </output>
                  </span>
                </label>
              {/if}

              {#if performanceSaveState.status !== "saved"}
                <p
                  class:error={performanceSaveState.status === "error"}
                  class="model-hint"
                  role={performanceSaveState.status === "error"
                    ? "alert"
                    : undefined}
                >
                  {saveStatusText(performanceSaveState)}
                </p>
              {/if}
            </fieldset>
          {/if}

          {#if captionSettings}
            <fieldset>
              <legend>Speaker labels</legend>
              <label class="checkbox-row">
                <input
                  type="checkbox"
                  checked={captionSettings.speakerLabelsEnabled}
                  onchange={(event) =>
                    updateSpeakerLabels(
                      (event.currentTarget as HTMLInputElement).checked,
                    )}
                />
                <span>
                  <strong>Experimental speaker labels</strong>
                  <small>High-resource mode. Shows low/medium confidence.</small
                  >
                </span>
              </label>
              {#if captionSaveState.status !== "saved"}
                <p
                  class:error={captionSaveState.status === "error"}
                  class="model-hint"
                  role={captionSaveState.status === "error"
                    ? "alert"
                    : undefined}
                >
                  {saveStatusText(captionSaveState)}
                </p>
              {/if}
            </fieldset>
          {/if}

          {#if transcriptSettings}
            <fieldset>
              <legend>Transcripts</legend>
              <label class="checkbox-row">
                <input
                  type="checkbox"
                  checked={transcriptSettings.savingEnabled}
                  onchange={(event) =>
                    updateTranscriptSaving(
                      (event.currentTarget as HTMLInputElement).checked,
                    )}
                />
                <span>
                  <strong>Save transcripts</strong>
                  <small>Off by default. Stored locally.</small>
                </span>
              </label>
              {#if transcriptSaveState.status !== "saved"}
                <p
                  class:error={transcriptSaveState.status === "error"}
                  class="model-hint"
                  role={transcriptSaveState.status === "error"
                    ? "alert"
                    : undefined}
                >
                  {saveStatusText(transcriptSaveState)}
                </p>
              {/if}
              <div class="transcript-history">
                <div class="section-row">
                  <strong>History</strong>
                  <button
                    type="button"
                    class="secondary-action small-action"
                    onclick={() => void refreshTranscriptSessions()}
                  >
                    Refresh
                  </button>
                </div>

                {#if transcriptSessions.length === 0}
                  <p class="model-hint">No saved transcripts yet.</p>
                {:else}
                  <div class="transcript-list">
                    {#each transcriptSessions as session}
                      <article class="transcript-session">
                        <div>
                          <strong>{session.sourceSummary}</strong>
                          <small>
                            {formatSessionTime(session.startedAtMs)} ·
                            {session.segmentCount} segments
                          </small>
                        </div>
                        <div class="export-actions">
                          {#each ["txt", "srt", "vtt", "json"] as format}
                            <button
                              type="button"
                              class="secondary-action small-action"
                              onclick={() =>
                                void exportTranscript(
                                  session,
                                  format as TranscriptExportFormat,
                                )}
                            >
                              {format.toUpperCase()}
                            </button>
                          {/each}
                        </div>
                      </article>
                    {/each}
                  </div>
                {/if}

                {#if transcriptHistoryState.status === "loading"}
                  <p class="model-hint">Working</p>
                {:else if transcriptHistoryState.status === "exported"}
                  <p class="model-hint">
                    Exported to {transcriptHistoryState.path}
                  </p>
                {:else if transcriptHistoryState.status === "error"}
                  <p class="model-hint error" role="alert">
                    {transcriptHistoryState.message}
                  </p>
                {/if}
              </div>
            </fieldset>
          {/if}

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
              Original line
              <span class="range-row">
                <input
                  type="range"
                  min="0.6"
                  max="1"
                  step="0.01"
                  value={draft.originalLineScale}
                  oninput={(event) =>
                    updateSetting("originalLineScale", numberInput(event))}
                />
                <output>{Math.round(draft.originalLineScale * 100)}%</output>
              </span>
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
              <p>
                <span
                  style={previewTextStyle(
                    draft,
                    Math.round(draft.fontSize * draft.originalLineScale),
                  )}
                >
                  Original caption
                </span>
                <span style={previewTextStyle(draft)}>
                  English translation
                </span>
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

  .app-version {
    margin: 0 auto 0 0;
    color: var(--fgColor-muted);
    font-size: 0.8rem;
    font-weight: 700;
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

  .section-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    color: var(--fgColor-default);
  }

  .small-action {
    min-height: 30px;
    padding: 0 10px;
    font-size: 0.78rem;
  }

  .transcript-history,
  .transcript-list,
  .transcript-session {
    display: grid;
    gap: 8px;
    min-width: 0;
  }

  .transcript-session {
    border: 1px solid var(--borderColor-muted);
    border-radius: var(--radius-default);
    padding: 10px;
    background: var(--bgColor-inset);
  }

  .transcript-session small {
    display: block;
    margin-top: 3px;
    color: var(--fgColor-muted);
    font-size: 0.76rem;
  }

  .export-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .placement-error {
    margin: 0;
    color: var(--danger-fgColor, oklch(74% 0.16 24));
    font-size: 0.84rem;
  }

  .model-status,
  .model-hint {
    margin: 0;
    color: var(--fgColor-muted);
    font-size: 0.84rem;
  }

  .model-status.model-ready {
    color: var(--button-primary-bgColor-rest);
  }

  .model-hint.error {
    color: var(--danger-fgColor, oklch(74% 0.16 24));
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
    display: grid;
    gap: 6px;
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
