| MIDI Control Change | MIDI Control Change Purpose | Implementated | Value | MIDI Control Change Description |
| --- | --- | --- | --- | --- |
| 0 | Bank Select (MSB) | No | 0-127 | Allows user to switch bank for patch selection. Program change used with Bank Select. MIDI can access 16,384 patches per MIDI channel. |
| 1 | Modulation Wheel (MSB) | No | 0-127 | Generally this CC controls a vibrato effect (pitch, loudness, brighness). What is modulated is based on the patch. |
| 2 | Breath Controller (MSB) | No | 0-127 | Oftentimes associated with aftertouch messages. It was originally intended for use with a breath MIDI controller in which blowing harder produced higher MIDI control values. It can be used for modulation as well. |
| 3 | Undefined (MSB) | No | 0-127 |  |
| 4 | Foot Pedal (MSB) | No | 0-127 | Often used with aftertouch messages. It can send a continuous stream of values based on how the pedal is used. |
| 5 | Portamento Time (MSB) | No | 0-127 | Controls portamento rate to slide between 2 notes played subsequently. |
| 6 | Data Entry (MSB) | No | 0-127 | Controls Value for NRPN or RPN parameters. |
| 7 | Volume (MSB) | No | 0-127 | Controls the volume of the channel. |
| 8 | Balance (MSB) | No | 0-127 | Controls the left and right balance, generally for stereo patches. A value of 64 equals the center. |
| 9 | Undefined (MSB) | No | 0-127 |  |
| 10 | Pan (MSB) | No | 0-127 | Controls the left and right balance, generally for mono patches. A value of 64 equals the center. |
| 11 | Expression (MSB) | No | 0-127 | Expression is a percentage of volume (CC7). |
| 12 | Effect Controller 1 (MSB) | No | 0-127 | Usually used to control a parameter of an effect within the synth or workstation. |
| 13 | Effect Controller 2 (MSB) | No | 0-127 | Usually used to control a parameter of an effect within the synth or workstation. |
| 14 | Undefined (MSB) | No | 0-127 |  |
| 15 | Undefined (MSB) | No | 0-127 |  |
| 16 | - 19 | General Purpose (MSB) | No | 0-127 |  |
| 20 | - 31 | Undefined (MSB) | No | 0-127 |  |
| 32 | - 63 | LSB Controller for 0-3 | No | 0-127 |  |
| 64 | Damper Pedal on/off | No | ≤63 off, ≥64 on | On/off switch that controls sustain pedal. Nearly every synth will react to CC 64. (See also Sostenuto CC 66) |
| 65 | Portamento on/off | No | ≤63 off, ≥64 on | On/off switch |
| 66 | Sostenuto Pedal on/off | No | ≤63 off, ≥64 on | On/off switch – Like the Sustain controller (CC 64), However, it only holds notes that were “On” when the pedal was pressed. People use it to “hold” chords” and play melodies over the held chord. |
| 67 | Soft Pedal on/off | No | ≤63 off, ≥64 on | On/off switch – Lowers the volume of notes played. |
| 68 | Legato FootSwitch | No | ≤63 off, ≥64 on | On/off switch – Turns Legato effect between 2 subsequent notes on or off. |
| 69 | Hold 2 | No | ≤63 off, ≥64 on | Another way to “hold notes” (see MIDI CC 64 and MIDI CC 66). However notes fade out according to their release parameter rather than when the pedal is released. |
| 70 | Sound Controller 1 | No | 0-127 | Usually controls the way a sound is produced. Default = Sound Variation. |
| 71 | Sound Controller 2 | No | 0-127 | Allows shaping the Voltage Controlled Filter (VCF). Default = Resonance also (Timbre or Harmonics) |
| 72 | Sound Controller 3 | No | 0-127 | Controls release time of the Voltage controlled Amplifier (VCA). Default = Release Time. |
| 73 | Sound Controller 4 | No | 0-127 | Controls the “Attack’ of a sound. The attack is the amount of time it takes for the sound to reach maximum amplitude. |
| 74 | Sound Controller 5 | No | 0-127 | Controls VCFs cutoff frequency of the filter. |
| 75 | Sound Controller 6 | No | 0-127 | Generic – Some manufacturers may use to further shave their sounds. |
| 76 | Sound Controller 7 | No | 0-127 | Generic – Some manufacturers may use to further shave their sounds. |
| 77 | Sound Controller 8 | No | 0-127 | Generic – Some manufacturers may use to further shave their sounds. |
| 78 | Sound Controller 9 | No | 0-127 | Generic – Some manufacturers may use to further shave their sounds. |
| 79 | Sound Controller 10 | No | 0-127 | Generic – Some manufacturers may use to further shave their sounds. |
| 80 | General Purpose MIDI CC Controller | No | 0-127 | Decay Generic or on/off switch ≤63 off, ≥64 on |
| 81 | General Purpose MIDI CC Controller | No | 0-127 | Hi-Pass Filter Frequency or Generic on/off switch ≤63 off, ≥64 on |
| 82 | General Purpose MIDI CC Controller | No | 0-127 | Generic on/off switch ≤63 off, ≥64 on |
| 83 | General Purpose MIDI CC Controller | No | 0-127 | Generic on/off switch ≤63 off, ≥64 on |
| 84 | Portamento CC Control | No | 0-127 | Controls the amount of Portamento. |
| 85 | - 87 | Undefined— | No |  |
| 88 | High Resolution Velocity Prefix | No | 0-127 | Extends the range of possible velocity values |
| 89 | & 90 | Undefined— | No |  |
| 91 | Effect 1 Depth | No | 0-127 | Usually controls reverb send amount |
| 92 | Effect 2 Depth | No | 0-127 | Usually controls tremolo amount |
| 93 | Effect 3 Depth | No | 0-127 | Usually controls chorus amount |
| 94 | Effect 4 Depth | No | 0-127 | Usually controls detune amount |
| 95 | Effect 5 Depth | No | 0-127 | Usually controls phaser amount |
| 96(+1) | Data Increment | No | N/A | Usually used to increment data for RPN and NRPN messages. |
| 97(-1) | Data Decrement | No | N/A | Usually used to decrement data for RPN and NRPN messages. |
| 98 | Non-Registered Parameter Number LSB (NRPN) | No | 0-127 | For controllers 6, 38, 96, and 97, it selects the NRPN parameter. |
| 99 | Non-Registered Parameter Number MSB (NRPN) | No | 0-127 | For controllers 6, 38, 96, and 97, it selects the NRPN parameter. |
| 100 | Registered Parameter Number LSB (RPN) | No | 0-127 | For controllers 6, 38, 96, and 97, it selects the RPN parameter. |
| 101 | Registered Parameter Number MSB (RPN) | No | 0-127 | For controllers 6, 38, 96, and 97, it selects the RPN parameter. |
| 102 | - 119 | Undefined— | No |  |
| 120 to 127 | Channel Mode Messages | No |  |  |
| 120 | All Sound Off | No | 0 | Mutes all sound. It does so regardless of release time or sustain. (See MIDI CC 123) |
| 121 | Reset All Controllers | No | 0 | It will reset all controllers to their default. |
| 122 | Local on/off Switch | No | 0 off, 127 on | Turns internal connection of a MIDI keyboard or workstation, etc. on or off. If you use a computer, you will most likely want local control off to avoid notes being played twice. Once locally and twice when the note is sent back from the computer to your keyboard. |
| 123 | All Notes Off | No | 0 | Mutes all sounding notes. Release time will still be maintained, and notes held by sustain will not turn off until sustain pedal is depressed. |
| 124 | Omni Mode Off | No | 0 | Sets to “Omni Off” mode. |
| 125 | Omni Mode On | No | 0 | Sets to “Omni On” mode. |
| 126 | Mono Mode→ | No | Sets device mode to Monophonic. The value equals the number of channels, or 0 if the number of channels equals the number of voices in the receiver. |  |
| 127 | Poly Mode | No | 0 | Sets device mode to Polyphonic. |