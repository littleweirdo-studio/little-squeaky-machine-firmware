# Flash storage Key assignment

Room voor 9 x 99 patches, wavetable forms and sampler samples

## Key layout

Key = **AAA**_BCCu16

- AAA = 001 = patch
- AAA = 002 = wavetable forms
- AAA = 003 = sampler samples
- B = 0..9 = bank 0 .. 9
- CC = 0..99 = slot 0 ... 99

## Examples

| Key | Description | Key Example | Key Description |
|----------|----------|----------|----------|
| 001_0XX | Patches bank 0 | 001_000 | Patch bank 0 - patch 0 |
| 001_?XX | Patches bank ? | 001_?50 | Patch bank ? - patch 50 |
| 001_9XX | Patches bank 9 | 001_911 | Patch bank 99 - patch 11 |
| 002_XXX | Wavetable Synthesizer samples| 002_000 | Wavetable Synthesizer samples bank 0 - waveform 0 |
| 003_XXX | Sampler samples | 003_010 | Sampler samples bank 0 - sample 10 |