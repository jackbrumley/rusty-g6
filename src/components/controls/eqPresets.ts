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
