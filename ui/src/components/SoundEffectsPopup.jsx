import { useEffect, useRef, useState } from "react";
import { ChevronLeft, X, Music, Film, Gamepad2 } from "lucide-react";
import Equalizer from "./Equalizer";
import {
  SECTION_CUSTOM,
  SECTION_DEFAULT,
  SECTION_SPATIAL,
  deriveActiveSection,
  devicePresetId,
  hasCustom,
  hasDefault,
  hasSpatial,
  presetOptions,
} from "../lib/soundEffects";

/** How long an optimistic selection wins over the polled device state. */
const PENDING_GRACE_MS = 2000;

/* ── Radio circle indicator ──────────────────────────────────────── */

function RadioCircle({ active }) {
  return (
    <span className={"flex h-5 w-5 items-center justify-center rounded-full ring-2 transition " +
      (active ? "bg-brand ring-brand" : "ring-white/20")}>
      {active && <span className="h-2 w-2 rounded-full bg-white" />}
    </span>
  );
}

/* ── Spatial Audio section ────────────────────────────────────────── */

// Icon + nicer label per known SpatialAudioMode variant. Unknown variants still
// render, using the device's own localized label.
const SPATIAL_MODE_META = {
  Music: { label: "Music Mode", Icon: Music },
  Movie: { label: "Movie Mode", Icon: Film },
  Gaming: { label: "Gaming Mode", Icon: Gamepad2 },
};

function SpatialAudioSection({ s, send, expanded, onToggle }) {
  const modeSetting = s.spatialAudioMode;
  const currentMode = modeSetting?.value || "Music";
  const opts = modeSetting?.setting?.options || [];
  const localized = modeSetting?.setting?.localizedOptions || [];

  const modes = (opts.length > 0 ? opts : Object.keys(SPATIAL_MODE_META)).map((id, i) => ({
    id,
    label: SPATIAL_MODE_META[id]?.label || localized[i] || id,
    Icon: SPATIAL_MODE_META[id]?.Icon || Music,
  }));

  const [localMode, setLocalMode] = useState(currentMode);
  const lastEdit = useRef(0);

  useEffect(() => {
    if (Date.now() - lastEdit.current < 500) return;
    setLocalMode(currentMode);
  }, [currentMode]);

  const setMode = (mode) => {
    lastEdit.current = Date.now();
    setLocalMode(mode);
    send("spatialAudioMode", mode);
  };

  return (
    <div className="rounded-2xl bg-surface ring-1 ring-white/[0.04] overflow-hidden">
      <button
        type="button"
        onClick={onToggle}
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
        <RadioCircle active={expanded} />
      </button>

      {expanded && (
        <div className="grid grid-cols-3 gap-2 px-4 pb-4 pt-1">
          {modes.map(({ id, label, Icon }) => {
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

function DefaultPresetCard({ s, send, expanded, onToggle }) {
  // NOTE: presetEqualizerProfile nests its options under `select`, not `setting`.
  const options = presetOptions(s);
  const currentPreset = devicePresetId(s);
  const currentLabel = options.find((o) => o.id === currentPreset)?.label ?? currentPreset;
  const volAdj = s.volumeAdjustments;
  const bands = volAdj?.value || [];
  const min = volAdj?.setting?.min ?? -6;
  const max = volAdj?.setting?.max ?? 6;

  const applyPreset = (id) => {
    send("presetEqualizerProfile", id);
  };

  return (
    <div className="rounded-2xl bg-surface ring-1 ring-white/[0.04] overflow-hidden">
      <button
        type="button"
        onClick={onToggle}
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
            {currentLabel && (
              <span className="ml-2 text-[12px] text-muted-foreground">{currentLabel}</span>
            )}
          </div>
        </div>
        <RadioCircle active={expanded} />
      </button>

      {expanded && (
        <div className="px-4 pb-4 pt-1 space-y-3">
          {bands.length > 0 && (
            <div className="flex justify-center">
              <CurvePreview bands={bands} min={min} max={max} />
            </div>
          )}
          <div className="grid grid-cols-2 gap-2">
            {options.map(({ id, label }) => (
              <button
                key={id}
                type="button"
                onClick={() => { if (id !== currentPreset) applyPreset(id); }}
                className={
                  "truncate rounded-full px-3 py-2 text-[12px] font-medium transition " +
                  (id === currentPreset
                    ? "bg-brand text-brand-foreground"
                    : "bg-surface-elevated text-foreground/80 hover:bg-white/[0.08]")
                }
              >
                {label}
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

function CustomEQCard({ customPresetName, expanded, onToggle }) {
  return (
    <div className="rounded-2xl bg-surface ring-1 ring-white/[0.04] overflow-hidden">
      <button
        type="button"
        onClick={onToggle}
        className="w-full flex items-center justify-between px-4 py-3 transition hover:bg-white/[0.02]"
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
            {customPresetName && (
              <span className="ml-2 text-[12px] text-muted-foreground">{customPresetName}</span>
            )}
          </div>
        </div>
        <RadioCircle active={expanded} />
      </button>
    </div>
  );
}

/* ── Main popup ───────────────────────────────────────────────────── */

export default function SoundEffectsPopup({
  d,
  s,
  send,
  onClose,
  eqPresets = [],
  onEqPresetsChanged,
  customPresetName,
}) {
  const [showEqualizer, setShowEqualizer] = useState(false);

  const showSpatial = hasSpatial(s);
  const showDefault = hasDefault(s);
  const showCustom = hasCustom(s);

  // The selected mode is *derived* from the live snapshot, never stored. The
  // device enforces mutual exclusion itself, so this can't drift — and it
  // survives closing/reopening the popup because there is nothing to lose.
  const deviceSection = deriveActiveSection(s);

  // Optimistic override so a tap feels instant instead of waiting for the next
  // 900ms poll. Expires on its own, and early once the device agrees.
  const [pending, setPending] = useState(null);
  const pendingAt = useRef(0);
  useEffect(() => {
    if (!pending) return;
    if (deviceSection === pending) {
      setPending(null);
      return;
    }
    const left = PENDING_GRACE_MS - (Date.now() - pendingAt.current);
    if (left <= 0) {
      setPending(null);
      return;
    }
    const t = setTimeout(() => setPending(null), left);
    return () => clearTimeout(t);
  }, [pending, deviceSection]);

  const activeSection = pending ?? deviceSection;

  // Remember the last built-in preset so re-selecting "Default" restores it
  // instead of snapping back to the first option in the list.
  const lastPresetId = useRef(null);
  const currentPresetId = devicePresetId(s);
  useEffect(() => {
    if (currentPresetId != null) lastPresetId.current = currentPresetId;
  }, [currentPresetId]);

  const selectSection = (section) => {
    if (activeSection === section) {
      // Second tap on the already-active custom card drills into the editor.
      if (section === SECTION_CUSTOM) setShowEqualizer(true);
      return;
    }

    // No need to switch the previous mode off: writing any equalizer value
    // clears spatial audio device-side, and enabling spatial audio supersedes
    // the equalizer. Explicitly "resetting" here used to flatten the user's
    // curve, which destroyed data for no benefit.
    if (section === SECTION_SPATIAL) {
      send("spatialAudio", "true");
    } else if (section === SECTION_DEFAULT) {
      const id = lastPresetId.current ?? presetOptions(s)[0]?.id;
      if (!id) return;
      send("presetEqualizerProfile", id);
    } else if (section === SECTION_CUSTOM) {
      // Re-tag the curve that is already playing as the custom profile, so the
      // sound does not change — the user can then edit it.
      const volAdj = s.volumeAdjustments;
      if (!volAdj || volAdj.readOnly) return;
      const bands = volAdj.value || [];
      if (bands.length === 0) return;
      send("volumeAdjustments", bands.join(","));
    }

    pendingAt.current = Date.now();
    setPending(section);
  };

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
          <Equalizer
            setting={s.volumeAdjustments}
            preset={s.presetEqualizerProfile}
            send={send}
            model={d.model}
            defaultOpen
            customPresets={eqPresets}
            onPresetsChanged={onEqPresetsChanged}
          />
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

      {/* Content — radio group: only one section expanded */}
      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        {showSpatial && (
          <SpatialAudioSection s={s} send={send}
            expanded={activeSection === SECTION_SPATIAL}
            onToggle={() => selectSection(SECTION_SPATIAL)}
          />
        )}
        {showDefault && (
          <DefaultPresetCard s={s} send={send}
            expanded={activeSection === SECTION_DEFAULT}
            onToggle={() => selectSection(SECTION_DEFAULT)}
          />
        )}
        {showCustom && (
          <CustomEQCard
            customPresetName={customPresetName}
            expanded={activeSection === SECTION_CUSTOM}
            onToggle={() => selectSection(SECTION_CUSTOM)}
          />
        )}
      </div>
    </div>
  );
}
