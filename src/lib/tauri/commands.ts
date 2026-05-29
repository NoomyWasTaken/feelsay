import { invoke } from "@tauri-apps/api/core";
import type { AudioLevelEvent } from "$lib/domain/audio-meter";
import type { AppSettings, AppStatus } from "$lib/domain/settings";
import type {
  AudioSource,
  PlatformCapabilities,
  SourcePreview,
} from "$lib/domain/source-selection";

export function getAppStatus(): Promise<AppStatus> {
  return invoke<AppStatus>("get_app_status");
}

export function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export function saveSettings(settings: AppSettings): Promise<AppSettings> {
  return invoke<AppSettings>("save_settings", { settings });
}

export function showOverlay(): Promise<void> {
  return invoke<void>("show_overlay");
}

export function destroyOverlay(): Promise<void> {
  return invoke<void>("destroy_overlay");
}

export function hideOverlay(): Promise<void> {
  return destroyOverlay();
}

export function startAudioMeter(sourceIds: string[]): Promise<void> {
  return invoke<void>("start_audio_meter", { sourceIds });
}

export function stopAudioMeter(): Promise<void> {
  return invoke<void>("stop_audio_meter");
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
