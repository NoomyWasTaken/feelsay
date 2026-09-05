export type PerformancePreset =
  | "low_resource"
  | "balanced"
  | "accuracy"
  | "custom";

export type PerformanceSettings = {
  preset: PerformancePreset;
  minTranscribeSeconds: number;
  maxTranscribeSeconds: number;
  transcribeIntervalMs: number;
  asrQueueCapacity: number;
  vadSpeechLevelThreshold: number;
  vadSilenceLevelThreshold: number;
  vadSpeechFrames: number;
  vadSilenceFrames: number;
  resourceImpact: string;
};
