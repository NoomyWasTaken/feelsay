export type AudioMeterStatus = "idle" | "starting" | "active";

export type AudioLevelEvent = {
  level: number;
  status: AudioMeterStatus;
  sourceIds: string[];
  isMock: true;
};
