/// Graphic EQ presets: band gains in dB, in the order of
/// `EQ_BAND_FREQUENCIES` (31 Hz ... 16 kHz). Rusty G6's own curves.
export interface EqPreset {
  id: string;
  name: string;
  gainsDb: number[];
}

export const EQ_PRESETS: EqPreset[] = [
  { id: "flat", name: "Flat", gainsDb: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
  { id: "bass-boost", name: "Bass Boost", gainsDb: [6, 5, 4, 2, 0, 0, 0, 0, 0, 0] },
  { id: "bass-reducer", name: "Bass Reducer", gainsDb: [-6, -5, -4, -2, 0, 0, 0, 0, 0, 0] },
  { id: "treble-boost", name: "Treble Boost", gainsDb: [0, 0, 0, 0, 0, 1, 2, 4, 5, 6] },
  { id: "treble-reducer", name: "Treble Reducer", gainsDb: [0, 0, 0, 0, 0, -1, -2, -4, -5, -6] },
  { id: "vocal", name: "Vocal", gainsDb: [-2, -2, -1, 0, 2, 4, 4, 2, 0, -1] },
  { id: "rock", name: "Rock", gainsDb: [4, 3, 2, 0, -1, -1, 1, 2, 3, 4] },
  { id: "pop", name: "Pop", gainsDb: [-1, 0, 2, 3, 3, 2, 0, -1, -1, -1] },
  { id: "electronic", name: "Electronic", gainsDb: [5, 4, 2, 0, -1, 0, 1, 2, 4, 5] },
  { id: "movie", name: "Movie", gainsDb: [3, 2, 1, 0, 0, 1, 2, 1, 0, 0] },
  { id: "footsteps", name: "Footsteps (games)", gainsDb: [-3, -3, -2, -1, 0, 2, 4, 4, 2, 0] },
];

/// The preset whose curve matches `gainsDb`, if any.
export function matchingPreset(gainsDb: number[]): EqPreset | undefined {
  return EQ_PRESETS.find((preset) =>
    preset.gainsDb.every((gainDb, band) => Math.abs(gainDb - gainsDb[band]) < 0.01)
  );
}

/// Creative's mic EQ presets, as captured (band gains in dB, lowest band
/// first; linuxblaster_control `MIC_EQ_PRESETS`). Creative names them only
/// by number, so each description says what its curve does.
export const MIC_EQ_PRESETS: { id: string; name: string; description: string; gainsDb: number[] }[] = [
  { id: "1", name: "Preset 1", description: "Less bass, more clarity.", gainsDb: [-3, -4, 0, 2, 3, -3, 4, 5] },
  { id: "2", name: "Preset 2", description: "Less bass, a little more clarity.", gainsDb: [-3, -4, 0, 2, 4, -2, 2, 4] },
  { id: "3", name: "Preset 3", description: "A fuller mid-range voice.", gainsDb: [-2, -3, 3, 4, 4, -4, 3, 2] },
  { id: "4", name: "Preset 4", description: "Less bass and rumble.", gainsDb: [-3, -5, 0, 4, 0, -3, 0, 0] },
  { id: "5", name: "Preset 5", description: "A fuller voice with softer treble.", gainsDb: [-2, -3, 2, 4, 4, 0, -3, 2] },
  { id: "6", name: "Preset 6", description: "Much brighter, with less bass.", gainsDb: [-5, -4, -2, 0, 3, 4, 6, 7] },
  { id: "7", name: "Preset 7", description: "More bass and treble, fewer mids.", gainsDb: [0, 3, -2, -4, -4, -2, 5, 7] },
  { id: "8", name: "Preset 8", description: "A little brighter.", gainsDb: [0, 0, 2, 2, 3, -4, 2, 4] },
  { id: "9", name: "Preset 9", description: "Softer upper mids.", gainsDb: [0, 0, 2, 2, -2, 0, -4, 4] },
  { id: "10", name: "Preset 10", description: "A brighter upper range.", gainsDb: [0, 2, -2, 0, 3, 5, 6, 5] },
  { id: "dm1", name: "DM 1", description: "A strong boost across the range.", gainsDb: [0, 8, 0, 12, 12, 4, 8, 10] },
];

/// The mic EQ preset whose curve matches `gainsDb`, if any.
export function matchingMicPreset(gainsDb: number[]) {
  return MIC_EQ_PRESETS.find((preset) =>
    preset.gainsDb.every((gainDb, band) => Math.abs(gainDb - gainsDb[band]) < 0.01)
  );
}
