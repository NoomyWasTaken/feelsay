# Local Translation Engine

P3-022 adds a local text-translation layer after ASR. Whisper remains the ASR engine. Whisper's built-in translation still handles English target translation, and non-English target languages use a separate text translation adapter.

## Current Adapter

The current adapter is Argos Translate through its local CLI:

```powershell
argos-translate --from-lang en --to-lang es "Hello world"
```

Argos is used because it is offline, open source, supports installable language packages, and exposes a stable command-line interface. Feelsay does not bundle Python, Argos, or translation packages in this PBI.

## Setup

Install Argos Translate and the language pair outside Feelsay:

```powershell
pip install argostranslate
argospm update
argospm search --from-lang en --to-lang es
argospm install translate-en_es
```

Then open Feelsay Settings -> Translation:

- Set `From` to the source language.
- Set `To` to the target language.
- Keep `argos-translate` if it is on PATH, or enter the full executable path.
- Click `Test translation`.

## Runtime Flow

1. Audio capture produces PCM frames.
2. ASR produces original text.
3. If target language is English, Whisper translation can still be used.
4. If target language is not English, the translation worker calls the Argos CLI with the saved source and target language.
5. Captions and transcripts receive the translated text.

This keeps translation separate from capture and ASR.

## Evaluated Options

- Argos Translate: best current fit for a local CLI adapter and installable packages.
- Marian NMT: strong native translation engine candidate for a future bundled worker.
- Bergamot Translator: strong browser/native local translation candidate, but more integration work.

## Limitations

- Real translation only works when Argos Translate and the required language package are installed locally.
- Source language auto-detection is not implemented for text translation. Choose a source language for non-English target translation.
- Model install/download management is still external to Feelsay.
- No cloud translation is used.
