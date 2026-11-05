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

## Control change

When sending a control change, the corresponding parameters of LttL Weirdo will changed. 

| Program Number | Description | Value |
| --- | --- |  --- |
| 0x07 | Main volume (0 to MAX) | 0 - 127 |
| 0x38 | Filter Cutoff Frequency | 0 - 127 |
| 0x39 | Filter Resonance (0 to MAX) | 0 - 127 |