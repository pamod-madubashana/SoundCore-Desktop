// ui/src/components/Equalizer.jsx
//
// Drop-in replacement for the inline <Equalizer /> in ui/src/App.jsx.
// Self-contained: only depends on react + lucide-react + the existing CSS
// tokens (--brand, bg-surface, bg-surface-elevated, text-muted-foreground).
//
// Usage in App.jsx:
//   import Equalizer from "./components/Equalizer";
//   ...
//   {s.volumeAdjustments && (
//     <Equalizer
//       setting={s.volumeAdjustments}
//       preset={s.presetEqualizerProfile}   // optional, may be undefined
//       send={send}
//     />
//   )}
//
// Everything user-facing lives in the small tables at the top of this file:
// tweak PRESETS / labels there instead of editing JSX.

import { useEffect, useMemo, useRef, useState } from "react";
import { ChevronDown, RotateCcw, SlidersHorizontal } from "lucide-react";

/* ------------------------------------------------------------------ config */

const SYNC_GRACE_MS = 2500; // ignore device polls right after a local edit

// Fallback presets, used when the device exposes no presetEqualizerProfile
// setting. Values are "gain shapes" resampled to however many bands the
// device reports, so this works for 5-band and 8-band hardware alike.
const PRESETS = {
  Flat: [0, 0, 0, 0, 0],
  "Bass Boost": [6, 4, 1, 0, 0],
  "Bass Reducer": [-6, -4, -1, 0, 0],
  Vocal: [-2, 0, 4, 3, 0],
  Acoustic: [3, 1, 2, 2, 3],
  Electronic: [5, 2, -1, 2, 4],
  "Treble Boost": [0, 0, 1, 4, 6],
  Podcast: [-3, 1, 4, 2, -1],
};

const hzLabel = (hz) => (hz >= 1000 ? `${hz / 1000}k` : String(hz));

/* ------------------------------------------------------------------ helpers */

// Resample a 5-point curve onto n bands with linear interpolation.
function resample(shape, n) {
  if (n === shape.length) return [...shape];
  return Array.from({ length: n }, (_, i) => {
    const t = (i / Math.max(1, n - 1)) * (shape.length - 1);
    const lo = Math.floor(t);
    const hi = Math.min(shape.length - 1, lo + 1);
    const a = shape[lo] ?? 0;
    const b = shape[hi] ?? a;
    return Math.round(a + (b - a) * (t - lo));
  });
}

const clamp = (v, min, max) => Math.min(max, Math.max(min, v));

/* ------------------------------------------------------------- band slider */

/** Samsung-style vertical band: light rail above the handle, accent stem below. */
function BandSlider({ hz, value, min, max, fd, disabled, onChange, onCommit }) {
  const ref = useRef(null);
  const dragging = useRef(false);

  const valueFromEvent = (e) => {
    const el = ref.current;
    if (!el) return value;
    const { top, height } = el.getBoundingClientRect();
    const ratio = 1 - clamp((e.clientY - top) / height, 0, 1);
    return Math.round(min + ratio * (max - min));
  };

  const pct = ((value - min) / (max - min)) * 100;

  const start = (e) => {
    if (disabled) return;
    dragging.current = true;
    e.currentTarget.setPointerCapture?.(e.pointerId);
    onChange(valueFromEvent(e));
  };
  const move = (e) => {
    if (dragging.current) onChange(valueFromEvent(e));
  };
  const end = () => {
    if (!dragging.current) return;
    dragging.current = false;
    onCommit();
  };
  const step = (delta) => {
    if (disabled) return;
    onChange(clamp(value + delta, min, max));
    onCommit();
  };

  const shown = value / 10 ** fd;

  return (
    <div className="flex min-w-0 flex-1 flex-col items-center">
      <span className="mb-1.5 text-[10px] font-medium tabular-nums text-muted-foreground">
        {shown > 0 ? "+" : ""}
        {shown.toFixed(fd)}
      </span>
      <div
        ref={ref}
        role="slider"
        tabIndex={disabled ? -1 : 0}
        aria-label={`${hzLabel(hz)} Hz`}
        aria-valuemin={min}
        aria-valuemax={max}
        aria-valuenow={value}
        onPointerDown={start}
        onPointerMove={move}
        onPointerUp={end}
        onPointerCancel={end}
        onKeyDown={(e) => {
          if (e.key === "ArrowUp") step(1);
          else if (e.key === "ArrowDown") step(-1);
        }}
        className={
          "relative h-40 w-full touch-none outline-none " +
          (disabled
            ? "opacity-50"
            : "cursor-ns-resize focus-visible:ring-1 focus-visible:ring-brand/60 focus-visible:ring-offset-0")
        }
      >
        {/* upper rail */}
        <span
          className="absolute left-1/2 top-0 w-[2px] -translate-x-1/2 rounded-full bg-white/45"
          style={{ height: `${100 - pct}%` }}
        />
        {/* accent stem */}
        <span
          className="absolute bottom-0 left-1/2 w-[2px] -translate-x-1/2 rounded-full bg-brand"
          style={{ height: `${pct}%` }}
        />
        {/* handle */}
        <span
          className="absolute left-1/2 h-[13px] w-[13px] -translate-x-1/2 rounded-full border-2 border-brand bg-surface"
          style={{ bottom: `calc(${pct}% - 6.5px)` }}
        />
      </div>
      <span className="mt-2 text-[10px] tabular-nums text-muted-foreground">{hzLabel(hz)}</span>
    </div>
  );
}

/* ------------------------------------------------------------ curve preview */

/** Tiny sparkline of the current curve, shown in the collapsed header. */
function CurvePreview({ bands, min, max }) {
  const w = 56;
  const h = 16;
  const points = bands
    .map((v, i) => {
      const x = (i / Math.max(1, bands.length - 1)) * w;
      const y = h - ((v - min) / (max - min)) * h;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
  return (
    <svg width={w} height={h} viewBox={`0 0 ${w} ${h}`} className="text-brand" aria-hidden>
      <polyline
        points={points}
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinejoin="round"
        strokeLinecap="round"
        opacity="0.85"
      />
    </svg>
  );
}

/* ------------------------------------------------------------------- main */

export default function Equalizer({ setting, preset, send, defaultOpen = false }) {
  const { bandHz, fractionDigits, min, max } = setting.setting;
  const readOnly = !!setting.readOnly;
  const fd = fractionDigits || 0;

  const [open, setOpen] = useState(defaultOpen);
  const [bands, setBands] = useState(setting.value || []);
  const lastEdit = useRef(0);

  // Sync from the device only when the content actually changed and we didn't
  // just edit, so a poll tick can't clobber the band being dragged.
  const incoming = (setting.value || []).join(",");
  useEffect(() => {
    if (Date.now() - lastEdit.current < SYNC_GRACE_MS) return;
    setBands(setting.value || []);
  }, [incoming]); // eslint-disable-line react-hooks/exhaustive-deps

  const touch = () => (lastEdit.current = Date.now());

  const commit = (next) => {
    touch();
    setBands(next);
    send("volumeAdjustments", next.join(","));
  };

  const setBand = (i, v) => {
    touch();
    setBands((b) => b.map((x, idx) => (idx === i ? v : x)));
  };
  const pushBands = () => {
    touch();
    setBands((b) => {
      send("volumeAdjustments", b.join(","));
      return b;
    });
  };

  // Prefer the device's own preset list; fall back to local shapes.
  const deviceOptions = preset?.setting?.options ?? null;
  const presetNames = useMemo(
    () => deviceOptions ?? Object.keys(PRESETS),
    [deviceOptions],
  );
  const activePreset = deviceOptions
    ? (preset?.value ?? "")
    : (Object.keys(PRESETS).find(
        (name) =>
          resample(PRESETS[name] ?? [], bandHz.length)
            .map((v) => clamp(v * 10 ** fd, min, max))
            .join(",") === bands.join(","),
      ) ?? "Custom");

  const applyPreset = (name) => {
    if (readOnly || !name) return;
    if (deviceOptions && preset) {
      touch();
      send(preset.id, name);
      return;
    }
    commit(
      resample(PRESETS[name] ?? [], bandHz.length).map((v) =>
        clamp(Math.round(v * 10 ** fd), min, max),
      ),
    );
  };
  const reset = () => !readOnly && commit(bandHz.map(() => 0));

  return (
    <div className="overflow-hidden rounded-2xl bg-surface ring-1 ring-white/[0.04]">
      {/* header / disclosure */}
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        aria-expanded={open}
        className="flex w-full items-center gap-2.5 px-3.5 py-3 text-left transition hover:bg-white/[0.02]"
      >
        <SlidersHorizontal className="h-4 w-4 flex-shrink-0 text-brand" />
        <span className="text-[14px] font-semibold text-brand">Equalizer</span>
        <span className="truncate text-[11px] text-muted-foreground">
          {readOnly ? "Read-only" : activePreset || "Custom"}
        </span>
        <span className="ml-auto flex items-center gap-2">
          {!open && <CurvePreview bands={bands} min={min} max={max} />}
          <ChevronDown
            className={
              "h-4 w-4 text-muted-foreground transition-transform " + (open ? "rotate-180" : "")
            }
          />
        </span>
      </button>

      {/* body */}
      <div
        className={
          "grid transition-[grid-template-rows,opacity] duration-300 ease-out " +
          (open ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0")
        }
      >
        <div className="overflow-hidden">
          <div className="space-y-4 px-3 pb-4 pt-1">
            {/* plot card */}
            <div className="relative rounded-2xl bg-surface-elevated px-2.5 pb-2 pt-3">
              {/* horizontal grid lines behind the bands */}
              <div
                className="pointer-events-none absolute inset-x-3 top-[30px] bottom-[30px]"
                aria-hidden
              >
                {Array.from({ length: 7 }).map((_, i) => (
                  <span
                    key={i}
                    className="absolute inset-x-0 h-px bg-white/[0.07]"
                    style={{ top: `${(i / 6) * 100}%` }}
                  />
                ))}
                <span className="absolute inset-x-0 top-0 h-px bg-white/25" />
              </div>
              <div className="relative flex items-end justify-between gap-0.5">
                {bandHz.map((hz, i) => (
                  <BandSlider
                    key={hz}
                    hz={hz}
                    value={bands[i] ?? 0}
                    min={min}
                    max={max}
                    fd={fd}
                    disabled={readOnly}
                    onChange={(v) => setBand(i, v)}
                    onCommit={pushBands}
                  />
                ))}
              </div>
            </div>

            {/* preset pills */}
            <div className="grid grid-cols-2 gap-2">
              {presetNames.map((name) => {
                const active = name === activePreset;
                return (
                  <button
                    key={name}
                    type="button"
                    disabled={readOnly}
                    onClick={() => applyPreset(name)}
                    className={
                      "truncate rounded-full px-3 py-2 text-[12px] font-medium transition disabled:opacity-50 " +
                      (active
                        ? "bg-brand text-brand-foreground"
                        : "bg-surface-elevated text-foreground/80 hover:bg-white/[0.08]")
                    }
                  >
                    {name}
                  </button>
                );
              })}
              <button
                type="button"
                onClick={reset}
                disabled={readOnly}
                className="col-span-2 flex items-center justify-center gap-1.5 rounded-full bg-surface-elevated px-3 py-2 text-[12px] font-medium text-muted-foreground transition hover:text-brand disabled:opacity-50"
              >
                <RotateCcw className="h-3.5 w-3.5" /> Reset to flat
              </button>
            </div>

            <p className="px-1 text-center text-[11px] text-muted-foreground">
              Tune the sound exactly the way you like it.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
