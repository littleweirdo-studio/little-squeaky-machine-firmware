# Flash storage Key assignment

Capacity: Supports up to 9 x 99 slots.
Partition: 8 MB maximum storage spaces.

Data Format:

- All wavetable forms and samples are in 16-bit resolution.
- Each wavetable form consists of 600 cycles.
- Sampler samples has a sample rate of 44100 hertz, resulting in a storage size of 88200 (88.2 KB/s) bytes per second.

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