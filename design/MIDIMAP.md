# MIDI Implementation

```
 ⡇ ⡇⢹⠁⢹⠁⡇ ⣏⡉ ⡇⢸⣏⡉⡇⣏⡱⡏⢱⡎⢱
 ⠧⠤⠇⠸ ⠸ ⠧⠤⠧⠤ ⠟⠻⠧⠤⠇⠇⠱⠧⠜⠣⠜
```

MIDI implementation of Little Weirdo on the LttL Squeaky Machine!

## Notes

Respond to the MIDI Note range C1 (`24`) till B7 (`107`)

## Program change

When sending a program change to LttL Weirdo, the corresponding Patch will be loaded and the sound will change instantly.

| Program Number | Patch | Description |
| --- | --- | --- |
| 0x00 | Bass Guitar | 1 Osc - Bass - Filtered|
| 0x01 | Bass Guitar Fourth | 2 Osc - Bass - Detuned (0-4) - Filtered |
| 0x02 | Bass Guitar Major | 3 Osc - Bass - Detuned (0-4-7) - Filtered |
| 0x03 | WeirdScience | 4 Osc - Square - Detuned (0-4-7-9) |

## Control change

When sending a control change, the corresponding parameters of LttL Weirdo will changed. 

| Program Number | Description | Value |
| --- | --- |  --- |
| 0x07 | Main volume (0 to MAX) | 0 - 127 |
| 0x38 | Filter Cutoff Frequency | 0 - 127 |
| 0x39 | Filter Resonance (0 to MAX) | 0 - 127 |