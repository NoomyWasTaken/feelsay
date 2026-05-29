export type SessionState = "idle" | "starting" | "listening" | "error";

export type AppStatus = {
  state: SessionState;
  message: string;
  selectedSource: string | null;
  overlayAvailable: boolean;
};

export type AppSettings = {
  overlay: OverlaySettings;
};

export type OverlaySettings = {
  width: number;
  height: number;
  opacity: number;
  fontFamily: string;
  fontSize: number;
  textColor: string;
  backgroundOpacity: number;
  showOriginalAndTranslation: boolean;
};
