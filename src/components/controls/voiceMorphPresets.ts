/// Voice morph presets: the eight values in `g6_spec::VoiceMorphValue`
/// order (formants 1-3 in Hz, pitch, envelope, pitch variability, quiver,
/// contour). The values are Creative's (G6 `PresetConfig.json`); the names
/// and descriptions are ours. Creative's "Neutral" is left out: it sounds
/// the same as voice morph off.
export interface VoiceMorphPreset {
  id: string;
  name: string;
  description: string;
  values: number[];
}

export const VOICE_MORPH_PRESETS: VoiceMorphPreset[] = [
  { id: "male", name: "Male", description: "Lower and deeper.", values: [400, 1400, 2000, 0.6, 1.05, 1, 0, 0] },
  { id: "female", name: "Female", description: "Higher and lighter.", values: [400, 1400, 2220, 2.37, 0.92, 1, 0, 0] },
  { id: "kid", name: "Kid", description: "Much higher, like a child.", values: [400, 1400, 2000, 3, 0.66, 1, 0, 0] },
  { id: "elderly", name: "Elderly", description: "Older, with a shaky voice.", values: [713, 1500, 1800, 1.4, 1.45, 1, 0.1818, 0] },
  { id: "deep", name: "Deep", description: "Very low and heavy.", values: [339.5443, 1362.496, 2046, 0.4291186, 1.201318, 1, 0, 0] },
  { id: "up-north", name: "Up north", description: "A sing-song lilt.", values: [400, 1400, 2000, 1, 1, -0.2066, 0, 0.264461] },
  { id: "unstable", name: "Unstable", description: "High, with a wandering pitch.", values: [400, 1400, 2000, 2.37, 0.92, 2.5, 0, 0] },
  { id: "emo", name: "Emo", description: "Flat and monotone.", values: [400, 1400, 2000, 1, 1, 0.2, 0, 0] },
  { id: "elf", name: "Elf", description: "A little higher and brighter.", values: [398, 1395, 2451, 1.11, 0.96, 1, 0, 0] },
  { id: "dwarf", name: "Dwarf", description: "Lower and gruff.", values: [294, 1527, 2055, 0.77, 0.97, 1, 0, 0] },
  { id: "demon", name: "Demon", description: "Low and menacing.", values: [453, 1544, 2600, 0.69, 2, 1, 0, 0] },
  { id: "infiltrator", name: "Infiltrator", description: "Thin, with a slight waver.", values: [200, 1370, 2000, 1.24, 1.06, 1, 0.13, 0] },
  { id: "brute", name: "Brute", description: "Low and heavy-set.", values: [383, 1428, 2550, 0.65, 1.31138, 1, 0.03, 0] },
  { id: "orc", name: "Orc", description: "Low and rough.", values: [468, 1321, 2200, 0.66, 1.5, 1, 0, 0] },
  { id: "marine", name: "Marine", description: "Low and steady.", values: [388, 1155, 1848, 0.8, 1.08, 0.54, 0, 0] },
  { id: "chipmunk", name: "Chipmunk", description: "Small and squeaky.", values: [400, 1400, 2000, 1, 0.601627, 1, 0, 0] },
  { id: "robot", name: "Robot", description: "Flat and mechanical.", values: [400, 1400, 2000, 1.394, 1, -0.00824, 0, 0] },
];

/// The preset whose values match the G6's, if any. The G6 stores 32-bit
/// floats, so the match allows a small rounding difference.
export function matchingVoiceMorphPreset(values: number[]) {
  return VOICE_MORPH_PRESETS.find((preset) =>
    preset.values.every((value, index) => Math.abs(value - values[index]) <= Math.abs(value) * 1e-4 + 1e-4)
  );
}
