export type TranscriptSettings = {
  savingEnabled: boolean;
};

export type TranscriptSegment = {
  startMs: number;
  endMs: number;
  originalText: string;
  translatedText: string | null;
  sourceLabel: string | null;
  speakerLabel: string | null;
  speakerConfidence: "low" | "medium" | null;
  isFinal: boolean;
};

export type TranscriptSessionSummary = {
  id: number;
  startedAtMs: number;
  endedAtMs: number | null;
  sourceSummary: string;
  segmentCount: number;
};

export type TranscriptExportFormat = "txt" | "srt" | "vtt" | "json";
