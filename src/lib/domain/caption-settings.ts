export type CaptionMode = "captions" | "translate" | "original_and_translation";

export type TranslationLanguage =
  | "auto"
  | "english"
  | "spanish"
  | "french"
  | "german"
  | "italian"
  | "portuguese"
  | "japanese"
  | "korean"
  | "chinese"
  | "arabic"
  | "polish"
  | "dutch"
  | "ukrainian";

export type CaptionSettings = {
  mode: CaptionMode;
  speakerLabelsEnabled: boolean;
  translationSourceLanguage: TranslationLanguage;
  translationTargetLanguage: TranslationLanguage;
};

export const translationLanguageOptions: Array<{
  value: TranslationLanguage;
  label: string;
}> = [
  { value: "auto", label: "Auto" },
  { value: "english", label: "English" },
  { value: "spanish", label: "Spanish" },
  { value: "french", label: "French" },
  { value: "german", label: "German" },
  { value: "italian", label: "Italian" },
  { value: "portuguese", label: "Portuguese" },
  { value: "japanese", label: "Japanese" },
  { value: "korean", label: "Korean" },
  { value: "chinese", label: "Chinese" },
  { value: "arabic", label: "Arabic" },
  { value: "polish", label: "Polish" },
  { value: "dutch", label: "Dutch" },
  { value: "ukrainian", label: "Ukrainian" },
];

export function formatTranslationLanguage(
  language: TranslationLanguage | undefined,
): string {
  return (
    translationLanguageOptions.find((option) => option.value === language)
      ?.label ?? "English"
  );
}
