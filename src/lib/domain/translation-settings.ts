export type TranslationEngineKind = "argos_cli";

export type TranslationSettings = {
  engine: TranslationEngineKind;
  executablePath: string;
};

export type TranslationEngineStatus = {
  engine: TranslationEngineKind;
  executablePath: string;
  isAvailable: boolean;
  message: string;
};

export type TranslationDiagnosticResult = {
  translatedText: string;
};
