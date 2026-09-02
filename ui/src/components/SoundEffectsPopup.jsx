import { useEffect, useRef, useState } from "react";
import { ChevronLeft, X, Music, Film, Gamepad2 } from "lucide-react";
import Equalizer from "./Equalizer";

const invoke = window.__TAURI__?.core?.invoke ?? (async () => {});

/* ── Spatial Audio section ────────────────────────────────────────── */

const SPATIAL_MODES = [
  { id: "Music", label: "Music Mode", Icon: Music },
  { id: "Movie", label: "Movie Mode", Icon: Film },
  { id: "Gaming", label: "Gaming Mode", Icon: Gamepad2 },
];

function SpatialAudioSection({ s, send }) {
  const enabled = !!s.spatialAudio?.value;
  const modeSetting = s.spatialAudioMode;
  const currentMode = modeSetting?.value || "Music";
  const opts = modeSetting?.setting?.options || [];

  const [localEnabled, setLocalEnabled] = useState(enabled);
  const [localMode, setLocalMode] = useState(currentMode);
  const lastEdit = useRef(0);

  useEffect(() => {
    if (Date.now() - lastEdit.current < 500) return;
    setLocalEnabled(enabled);
    setLocalMode(currentMode);
  }, [enabled, currentMode]);

  const toggleEnabled = () => {
    const next = !localEnabled;
    lastEdit.current = Date.now();
    setLocalEnabled(next);
    send("spatialAudio", String(next));
  };

  const setMode = (mode) => {
    lastEdit.current = Date.now();
    setLocalMode(mode);
    send("spatialAudioMode", mode);
  };

  return (
    <div className="rounded-2xl bg-surface ring-1 ring-white/[0.04] overflow-hidden">
      <button
        type="button"
        onClick={toggleEnabled}
        className="w-full flex items-center justify-between px-4 py-3 transition hover:bg-white/[0.02]"
      >
        <div className="flex items-center gap-3">
          <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-brand/10 text-brand">
            <svg className="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M9 18V5l12-2v13" /><circle cx="6" cy="18" r="3" /><circle cx="18" cy="16" r="3" />
            </svg>
          </div>
          <span className="text-[14px] font-medium text-foreground">Spatial Audio</span>
        </div>
        <span className={"relative h-[22px] w-[42px] rounded-full transition-colors flex-shrink-0 " + (localEnabled ? "bg-brand" : "bg-white/10")}>
          <span className={"absolute top-[3px] h-[16px] w-[16px] rounded-full bg-white shadow transition-all " + (localEnabled ? "left-[23px]" : "left-[3px]")} />
        </span>
      </button>

      {localEnabled && (
        <div className="grid grid-cols-3 gap-2 px-4 pb-4 pt-1">
          {SPATIAL_MODES.map(({ id, label, Icon }) => {
            if (opts.length > 0 && !opts.includes(id)) return null;
            const active = id === localMode;
            return (
              <button
                key={id}
                type="button"
                onClick={() => setMode(id)}
                className={
                  "flex flex-col items-center justify-center gap-2 rounded-xl py-3 transition-all " +
                  (active
                    ? "bg-brand text-brand-foreground ring-2 ring-brand/50"
                    : "bg-surface-elevated text-foreground/70 hover:bg-white/[0.06] ring-1 ring-white/[0.04]")
                }
              >
                <Icon className="h-5 w-5" />
                <span className="text-[11px] font-medium leading-tight">{label}</span>
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}

/* ── Default Preset card ──────────────────────────────────────────── */

function DefaultPresetCard({ s, send }) {
  const preset = s.presetEqualizerProfile;
  const options = preset?.setting?.options || [];
  const currentPreset = preset?.value || "";
  const volAdj = s.volumeAdjustments;
  const bands = volAdj?.value || [];
  const min = volAdj?.setting?.min ?? -6;
  const max = volAdj?.setting?.max ?? 6;
  const bandHz = volAdj?.setting?.bandHz || [];

  const [expanded, setExpanded] = useState(false);

  const applyPreset = (name) => {
    send("presetEqualizerProfile", name);
  };

  return (
    <div className="rounded-2xl bg-surface ring-1 ring-white/[0.04] overflow-hidden">
      <button
        type="button"
        onClick={() => setExpanded((o) => !o)}
        className="w-full flex items-center justify-between px-4 py-3 transition hover:bg-white/[0.02]"
      >
        <div className="flex items-center gap-3">
          <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-brand/10 text-brand">
            <svg className="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
              <rect x="3" y="3" width="7" height="7" rx="1" /><rect x="14" y="3" width="7" height="7" rx="1" />
              <rect x="3" y="14" width="7" height="7" rx="1" /><rect x="14" y="14" width="7" height="7" rx="1" />
            </svg>
          </div>
          <div className="text-left">
            <span className="text-[14px] font-medium text-foreground">Default</span>
            {currentPreset && (
              <span className="ml-2 text-[12px] text-muted-foreground">{currentPreset}</span>
            )}
          </div>
        </div>
        <ChevronLeft className={"h-4 w-4 text-muted-foreground transition-transform " + (expanded ? "-rotate-90" : "rotate-0")} />
      </button>

      {expanded && (
        <div className="px-4 pb-4 pt-1 space-y-3">
          {/* Mini curve */}
          {bands.length > 0 && (
            <div className="flex justify-center">
              <CurvePreview bands={bands} min={min} max={max} />
            </div>
          )}
          {/* Preset pills */}
          <div className="grid grid-cols-2 gap-2">
            {options.map((name) => (
              <button
                key={name}
                type="button"
                onClick={() => applyPreset(name)}
                className={
                  "truncate rounded-full px-3 py-2 text-[12px] font-medium transition " +
                  (name === currentPreset
                    ? "bg-brand text-brand-foreground"
                    : "bg-surface-elevated text-foreground/80 hover:bg-white/[0.08]")
                }
              >
                {name}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

/* ── Curve preview sparkline ──────────────────────────────────────── */

function CurvePreview({ bands, min, max }) {
  const w = 200;
  const h = 40;
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

/* ── Custom EQ card ───────────────────────────────────────────────── */

function CustomEQCard({ s, send, onOpenEqualizer }) {
  return (
    <button
      type="button"
      onClick={onOpenEqualizer}
      className="w-full rounded-2xl bg-surface ring-1 ring-white/[0.04] px-4 py-3 flex items-center justify-between transition hover:bg-white/[0.02]"
    >
      <div className="flex items-center gap-3">
        <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-brand/10 text-brand">
          <svg className="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
            <line x1="4" y1="21" x2="4" y2="14" /><line x1="4" y1="10" x2="4" y2="3" />
            <line x1="12" y1="21" x2="12" y2="12" /><line x1="12" y1="8" x2="12" y2="3" />
            <line x1="20" y1="21" x2="20" y2="16" /><line x1="20" y1="12" x2="20" y2="3" />
            <line x1="1" y1="14" x2="7" y2="14" /><line x1="9" y1="8" x2="15" y2="8" />
            <line x1="17" y1="16" x2="23" y2="16" />
          </svg>
        </div>
        <div className="text-left">
          <span className="text-[14px] font-medium text-foreground">Custom EQ</span>
          <span className="ml-2 text-[12px] text-muted-foreground">Custom</span>
        </div>
      </div>
      <ChevronLeft className="h-4 w-4 text-muted-foreground -rotate-180" />
    </button>
  );
}

/* ── Main popup ───────────────────────────────────────────────────── */

export default function SoundEffectsPopup({ d, s, send, onClose }) {
  const [showEqualizer, setShowEqualizer] = useState(false);

  if (showEqualizer) {
    return (
      <div className="absolute inset-0 z-50 flex flex-col bg-background animate-slide-up">
        <header className="flex items-center gap-3 px-4 py-3 border-b border-white/[0.05]">
          <button onClick={() => setShowEqualizer(false)}
            className="p-1 rounded-md text-muted-foreground hover:text-foreground hover:bg-white/5 transition">
            <ChevronLeft className="h-5 w-5" />
          </button>
          <h2 className="text-[15px] font-semibold text-foreground">Custom EQ</h2>
        </header>
        <div className="flex-1 overflow-y-auto p-4">
          <Equalizer setting={s.volumeAdjustments} preset={s.presetEqualizerProfile} send={send} model={d.model} defaultOpen />
        </div>
      </div>
    );
  }

  return (
    <div className="absolute inset-0 z-50 flex flex-col bg-background animate-slide-up">
      {/* Header */}
      <header className="flex items-center justify-between px-4 py-3 border-b border-white/[0.05]">
        <div className="flex items-center gap-3">
          <button onClick={onClose}
            className="p-1 rounded-md text-muted-foreground hover:text-foreground hover:bg-white/5 transition">
            <ChevronLeft className="h-5 w-5" />
          </button>
          <h2 className="text-[15px] font-semibold text-foreground">Sound Effects</h2>
        </div>
        <button onClick={onClose}
          className="p-1 rounded-md text-muted-foreground hover:text-foreground hover:bg-white/5 transition">
          <X className="h-4 w-4" />
        </button>
      </header>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        {s.spatialAudio && <SpatialAudioSection s={s} send={send} />}
        {s.presetEqualizerProfile && <DefaultPresetCard s={s} send={send} />}
        {s.volumeAdjustments && <CustomEQCard s={s} send={send} onOpenEqualizer={() => setShowEqualizer(true)} />}
      </div>
    </div>
  );
}
