import { invoke } from "@tauri-apps/api/core";
import type { AudioLevelEvent } from "$lib/domain/audio-meter";
import type {
  OverlayProfileId,
  OverlayPlacement,
  OverlaySettings,
  OverlaySettingsStore,
} from "$lib/domain/overlay-settings";
import type {
  AudioSource,
  PlatformCapabilities,
  SourcePreview,
} from "$lib/domain/source-selection";

export type SessionState = "idle" | "starting" | "listening" | "error";

export type AppStatus = {
  state: SessionState;
  message: string;
  selectedSource: string | null;
};

export function getAppStatus(): Promise<AppStatus> {
  return invoke<AppStatus>("get_app_status");
}

export function startAudioMeter(sourceIds: string[]): Promise<void> {
  return invoke<void>("start_audio_meter", { sourceIds });
}

export function stopAudioMeter(): Promise<void> {
  return invoke<void>("stop_audio_meter");
}

export function openCaptionWindow(): Promise<void> {
  return invoke<void>("open_caption_window");
}

export function closeCaptionWindow(): Promise<void> {
  return invoke<void>("close_caption_window");
}

export function isCaptionWindowOpen(): Promise<boolean> {
  return invoke<boolean>("is_caption_window_open");
}

export function getOverlaySettings(): Promise<OverlaySettings> {
  return invoke<OverlaySettings>("get_overlay_settings");
}

export function saveOverlaySettings(
  settings: OverlaySettings,
): Promise<OverlaySettings> {
  return invoke<OverlaySettings>("save_overlay_settings", { settings });
}

export function getOverlaySettingsStore(): Promise<OverlaySettingsStore> {
  return invoke<OverlaySettingsStore>("get_overlay_settings_store");
}

export function saveOverlaySettingsStore(
  store: OverlaySettingsStore,
): Promise<OverlaySettingsStore> {
  return invoke<OverlaySettingsStore>("save_overlay_settings_store", { store });
}

export function selectOverlaySettingsProfile(
  profileId: OverlayProfileId,
): Promise<OverlaySettingsStore> {
  return invoke<OverlaySettingsStore>("select_overlay_settings_profile", {
    profileId,
  });
}

export function startOverlayPlacement(
  settings: OverlaySettings,
): Promise<OverlayPlacement> {
  return invoke<OverlayPlacement>("start_overlay_placement", { settings });
}

export function getOverlayPlacement(): Promise<OverlayPlacement | null> {
  return invoke<OverlayPlacement | null>("get_overlay_placement");
}

export function stopOverlayPlacement(): Promise<void> {
  return invoke<void>("stop_overlay_placement");
}

export type { AudioLevelEvent };

export function getPlatformCapabilities(): Promise<PlatformCapabilities> {
  return invoke<PlatformCapabilities>("get_platform_capabilities");
}

export function listAvailableSources(): Promise<AudioSource[]> {
  return invoke<AudioSource[]>("list_available_sources");
}

export function getSourcePreviews(): Promise<SourcePreview[]> {
  return invoke<SourcePreview[]>("get_source_previews");
}

export function commandErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === "object" && error !== null) {
    const message = Reflect.get(error, "message");
    if (typeof message === "string") {
      return message;
    }
  }

  if (typeof error === "string" && error.length > 0) {
    return error;
  }

  return "The app could not complete that action.";
}
