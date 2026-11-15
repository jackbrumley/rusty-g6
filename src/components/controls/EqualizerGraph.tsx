import { useEffect, useRef, useState } from "preact/hooks";
import { EQ_BAND_FREQUENCIES, EQ_BAND_LIMIT_DB, EQ_GAIN_STEP_DB } from "../../types/g6";

interface EqualizerGraphProps {
  gainsDb: number[];
  dimmed: boolean;
  onSetBand: (band: number, gainDb: number) => void;
}

// Drawing area in SVG units; the SVG scales to its container.
const WIDTH = 600;
const HEIGHT = 240;
const PAD_LEFT = 40;
const PAD_RIGHT = 20;
const PAD_TOP = 20;
const PAD_BOTTOM = 30;
const PLOT_HEIGHT = HEIGHT - PAD_TOP - PAD_BOTTOM;
const GRID_DB = [12, 6, 0, -6, -12];
const PENDING_TIMEOUT_MS = 5000;

// The bands are an octave apart, so even spacing is a log-frequency axis.
const bandX = (band: number) =>
  PAD_LEFT + (band * (WIDTH - PAD_LEFT - PAD_RIGHT)) / (EQ_BAND_FREQUENCIES.length - 1);
const gainY = (gainDb: number) =>
  PAD_TOP + ((EQ_BAND_LIMIT_DB - gainDb) / (2 * EQ_BAND_LIMIT_DB)) * PLOT_HEIGHT;

const snapGain = (gainDb: number) => {
  const clamped = Math.max(-EQ_BAND_LIMIT_DB, Math.min(EQ_BAND_LIMIT_DB, gainDb));
  return Math.round(clamped / EQ_GAIN_STEP_DB) * EQ_GAIN_STEP_DB;
};

const formatDb = (gainDb: number) => `${gainDb > 0 ? "+" : ""}${gainDb.toFixed(1)} dB`;

/// A smooth curve through the band points (Catmull-Rom as cubic Béziers).
function curvePath(points: [number, number][]): string {
  let path = `M ${points[0][0]} ${points[0][1]}`;
  for (let i = 0; i < points.length - 1; i++) {
    const p0 = points[Math.max(i - 1, 0)];
    const p1 = points[i];
    const p2 = points[i + 1];
    const p3 = points[Math.min(i + 2, points.length - 1)];
    const c1x = p1[0] + (p2[0] - p0[0]) / 6;
    const c1y = p1[1] + (p2[1] - p0[1]) / 6;
    const c2x = p2[0] - (p3[0] - p1[0]) / 6;
    const c2y = p2[1] - (p3[1] - p1[1]) / 6;
    path += ` C ${c1x} ${c1y}, ${c2x} ${c2y}, ${p2[0]} ${p2[1]}`;
  }
  return path;
}

/// The EQ curve. Drag a band's handle up or down to set its gain; the gain
/// is sent when the drag ends. Handles also take arrow keys (0.5 dB),
/// Page Up/Down (3 dB) and a double-click (reset to 0 dB).
export function EqualizerGraph({ gainsDb, dimmed, onSetBand }: EqualizerGraphProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const [localGains, setLocalGains] = useState(gainsDb);
  const [dragBand, setDragBand] = useState<number | null>(null);
  // Gains sent to the device but not yet reported back in `gainsDb`. The
  // curve shows them meanwhile; otherwise it would flick back to the old
  // value when a drag ends and jump again when the device confirms.
  const pendingRef = useRef(new Map<number, number>());
  const gainsRef = useRef(gainsDb);
  gainsRef.current = gainsDb;

  const syncFromDevice = () => {
    const pending = pendingRef.current;
    for (const [band, gainDb] of pending) {
      if (gainsRef.current[band] === gainDb) {
        pending.delete(band);
      }
    }
    setLocalGains(gainsRef.current.map((gainDb, band) => pending.get(band) ?? gainDb));
  };

  useEffect(() => {
    if (dragBand === null) {
      syncFromDevice();
    }
  }, [gainsDb, dragBand]);

  // Where on the handle the drag started, in dB from its centre, so the
  // point doesn't jump to the pointer on the first move.
  const grabOffsetRef = useRef(0);

  const rawGainAtPointer = (event: PointerEvent): number | null => {
    const svg = svgRef.current;
    const matrix = svg?.getScreenCTM();
    if (!svg || !matrix) {
      return null;
    }
    const point = new DOMPoint(event.clientX, event.clientY).matrixTransform(matrix.inverse());
    return EQ_BAND_LIMIT_DB - ((point.y - PAD_TOP) / PLOT_HEIGHT) * 2 * EQ_BAND_LIMIT_DB;
  };

  const gainAtPointer = (event: PointerEvent): number | null => {
    const gainDb = rawGainAtPointer(event);
    return gainDb === null ? null : snapGain(gainDb - grabOffsetRef.current);
  };

  const setLocalBand = (band: number, gainDb: number) =>
    setLocalGains((gains) => gains.map((gain, index) => (index === band ? gainDb : gain)));

  const commitBand = (band: number, gainDb: number) => {
    setLocalBand(band, gainDb);
    if (gainDb === gainsDb[band]) {
      pendingRef.current.delete(band);
      return;
    }
    pendingRef.current.set(band, gainDb);
    onSetBand(band, gainDb);
    // A write that never shows up (it failed, with a toast) stops being
    // shown after a while, and the curve returns to the device's value.
    setTimeout(() => {
      if (pendingRef.current.get(band) === gainDb) {
        pendingRef.current.delete(band);
        syncFromDevice();
      }
    }, PENDING_TIMEOUT_MS);
  };

  const handlePointerDown = (band: number) => (event: PointerEvent) => {
    (event.currentTarget as Element).setPointerCapture(event.pointerId);
    const grabbedAt = rawGainAtPointer(event);
    grabOffsetRef.current = grabbedAt === null ? 0 : grabbedAt - localGains[band];
    setDragBand(band);
  };

  const handlePointerMove = (band: number) => (event: PointerEvent) => {
    if (dragBand !== band) {
      return;
    }
    const gainDb = gainAtPointer(event);
    if (gainDb !== null) {
      setLocalBand(band, gainDb);
    }
  };

  const handlePointerUp = (band: number) => (event: PointerEvent) => {
    if (dragBand !== band) {
      return;
    }
    setDragBand(null);
    const gainDb = gainAtPointer(event);
    commitBand(band, gainDb ?? localGains[band]);
  };

  const handleKeyDown = (band: number) => (event: KeyboardEvent) => {
    const steps: Record<string, number> = {
      ArrowUp: EQ_GAIN_STEP_DB,
      ArrowDown: -EQ_GAIN_STEP_DB,
      PageUp: 3,
      PageDown: -3,
    };
    const step = steps[event.key];
    if (step === undefined) {
      return;
    }
    event.preventDefault();
    commitBand(band, snapGain(localGains[band] + step));
  };

  const points = localGains.map((gainDb, band): [number, number] => [bandX(band), gainY(gainDb)]);
  const curve = curvePath(points);
  const zeroY = gainY(0);
  const fill = `${curve} L ${bandX(points.length - 1)} ${zeroY} L ${bandX(0)} ${zeroY} Z`;

  return (
    <svg
      ref={svgRef}
      class={`eq-graph ${dimmed ? "eq-graph-dimmed" : ""}`}
      viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
      role="group"
      aria-label="Equalizer curve"
    >
      {GRID_DB.map((gainDb) => (
        <g key={gainDb}>
          <line
            class={gainDb === 0 ? "eq-grid-zero" : "eq-grid"}
            x1={PAD_LEFT}
            x2={WIDTH - PAD_RIGHT}
            y1={gainY(gainDb)}
            y2={gainY(gainDb)}
          />
          <text class="eq-axis-label" x={PAD_LEFT - 8} y={gainY(gainDb) + 4} text-anchor="end">
            {gainDb > 0 ? `+${gainDb}` : gainDb}
          </text>
        </g>
      ))}

      {EQ_BAND_FREQUENCIES.map((frequency, band) => (
        <g key={frequency}>
          <line class="eq-grid" x1={bandX(band)} x2={bandX(band)} y1={PAD_TOP} y2={HEIGHT - PAD_BOTTOM} />
          <text class="eq-axis-label" x={bandX(band)} y={HEIGHT - 8} text-anchor="middle">
            {frequency}
          </text>
        </g>
      ))}

      <path class="eq-curve-fill" d={fill} />
      <path class="eq-curve" d={curve} />

      {points.map(([x, y], band) => (
        <g key={EQ_BAND_FREQUENCIES[band]}>
          {dragBand === band && (
            <text class="eq-drag-value" x={x} y={Math.max(y - 14, 12)} text-anchor="middle">
              {formatDb(localGains[band])}
            </text>
          )}
          <circle
            class="eq-handle"
            cx={x}
            cy={y}
            r={8}
            tabIndex={0}
            role="slider"
            aria-label={`${EQ_BAND_FREQUENCIES[band]} Hz`}
            aria-valuemin={-EQ_BAND_LIMIT_DB}
            aria-valuemax={EQ_BAND_LIMIT_DB}
            aria-valuenow={localGains[band]}
            aria-valuetext={formatDb(localGains[band])}
            onPointerDown={handlePointerDown(band)}
            onPointerMove={handlePointerMove(band)}
            onPointerUp={handlePointerUp(band)}
            onPointerCancel={() => setDragBand(null)}
            onKeyDown={handleKeyDown(band)}
            onDblClick={() => commitBand(band, 0)}
          >
            <title>{`${EQ_BAND_FREQUENCIES[band]} Hz: ${formatDb(localGains[band])}`}</title>
          </circle>
        </g>
      ))}
    </svg>
  );
}
