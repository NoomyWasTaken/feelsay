export type AudioMeterStatus = "idle" | "starting" | "active" | "error";

export type AudioLevelEvent = {
  level: number;
  status: AudioMeterStatus;
  sourceIds: string[];
  isMock: boolean;
  speechDetected: boolean;
  message?: string;
};
