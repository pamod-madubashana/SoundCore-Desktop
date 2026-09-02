// ui/src/components/Equalizer.jsx
//
// Self-contained equalizer with Samsung-style sliders, device presets,
// and custom user presets (save/rename/delete).

import { useEffect, useMemo, useRef, useState } from "react";
import { ChevronDown, RotateCcw, Save, Trash2, Pencil, Check, X, SlidersHorizontal } from "lucide-react";

const invoke = window.__TAURI__?.core?.invoke ?? (async () => {});

/* ------------------------------------------------------------------ config */

const SYNC_GRACE_MS = 2500;

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

/* ── API helpers ──────────────────────────────────────────────────── */

function loadPresets(model) {
  return invoke("list_eq_presets", { model }).catch(() => []);
}

function savePreset(name, bands, model) {
  return invoke("save_eq_preset", { name, bands, model });
}

function renamePreset(id, name) {
  return invoke("rename_eq_preset", { id, name });
}

function deletePreset(id) {
  return invoke("delete_eq_preset", { id });
}

/* ------------------------------------------------------------- band slider */

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
        onWheel={(e) => {
          if (disabled) return;
          e.preventDefault();
          e.stopPropagation();
          step(e.deltaY < 0 ? 1 : -1);
        }}
        onKeyDown={(e) => {
          if (e.key === "ArrowUp") { e.preventDefault(); step(1); }
          else if (e.key === "ArrowDown") { e.preventDefault(); step(-1); }
        }}
        className={
          "relative h-40 w-full touch-none outline-none " +
          (disabled
            ? "opacity-50"
            : "cursor-ns-resize focus-visible:ring-1 focus-visible:ring-brand/60 focus-visible:ring-offset-0")
        }
      >
        <span
          className="absolute left-1/2 top-0 w-[2px] -translate-x-1/2 rounded-full bg-white/45"
          style={{ height: `${100 - pct}%` }}
        />
        <span
          className="absolute bottom-0 left-1/2 w-[2px] -translate-x-1/2 rounded-full bg-brand"
          style={{ height: `${pct}%` }}
        />
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

/* ── Save dialog ──────────────────────────────────────────────────── */

function SaveDialog({ bands, model, onSaved, onCancel }) {
  const [name, setName] = useState("");
  const [saving, setSaving] = useState(false);

  const handleSave = async () => {
    if (!name.trim() || saving) return;
    setSaving(true);
    try {
      await savePreset(name.trim(), bands.join(","), model);
      onSaved();
    } catch {
      setSaving(false);
    }
  };

  return (
    <div className="flex items-center gap-2 rounded-xl bg-surface-elevated p-3 ring-1 ring-white/[0.06]">
      <input
        type="text"
        value={name}
        onChange={(e) => setName(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && handleSave()}
        placeholder="Preset name..."
        autoFocus
        className="flex-1 bg-transparent text-[13px] text-foreground placeholder:text-muted-foreground outline-none"
      />
      <button
        type="button"
        onClick={handleSave}
        disabled={!name.trim() || saving}
        className="flex h-7 w-7 items-center justify-center rounded-lg bg-brand text-brand-foreground transition disabled:opacity-50"
      >
        <Check className="h-3.5 w-3.5" />
      </button>
      <button
        type="button"
        onClick={onCancel}
        className="flex h-7 w-7 items-center justify-center rounded-lg bg-white/5 text-muted-foreground transition hover:text-foreground"
      >
        <X className="h-3.5 w-3.5" />
      </button>
    </div>
  );
}

/* ── Editable preset pill ─────────────────────────────────────────── */

function PresetPill({ name, active, isCustom, readOnly, onSelect, onRename, onDelete }) {
  const [editing, setEditing] = useState(false);
  const [newName, setNewName] = useState(name);

  if (editing) {
    return (
      <div className="flex items-center gap-1 rounded-full bg-surface-elevated ring-1 ring-brand/30 px-2 py-1">
        <input
          type="text"
          value={newName}
          onChange={(e) => setNewName(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && newName.trim()) { onRename(newName.trim()); setEditing(false); }
            if (e.key === "Escape") setEditing(false);
          }}
          autoFocus
          className="w-full bg-transparent text-[12px] font-medium text-foreground outline-none"
        />
        <button onClick={() => { if (newName.trim()) { onRename(newName.trim()); setEditing(false); } }}
          className="text-brand"><Check className="h-3 w-3" /></button>
        <button onClick={() => setEditing(false)}
          className="text-muted-foreground"><X className="h-3 w-3" /></button>
      </div>
    );
  }

  return (
    <div className="group relative">
      <button
        type="button"
        disabled={readOnly}
        onClick={() => onSelect(name)}
        className={
          "w-full truncate rounded-full px-3 py-2 text-[12px] font-medium transition disabled:opacity-50 " +
          (active
            ? "bg-brand text-brand-foreground"
            : "bg-surface-elevated text-foreground/80 hover:bg-white/[0.08]")
        }
      >
        {name}
      </button>
      {isCustom && !readOnly && (
        <div className="absolute -top-1 -right-1 hidden group-hover:flex items-center gap-0.5">
          <button onClick={(e) => { e.stopPropagation(); setEditing(true); }}
            className="flex h-5 w-5 items-center justify-center rounded-full bg-surface-elevated text-muted-foreground ring-1 ring-white/10 hover:text-brand">
            <Pencil className="h-2.5 w-2.5" />
          </button>
          <button onClick={(e) => { e.stopPropagation(); onDelete(); }}
            className="flex h-5 w-5 items-center justify-center rounded-full bg-surface-elevated text-muted-foreground ring-1 ring-white/10 hover:text-red-400">
            <Trash2 className="h-2.5 w-2.5" />
          </button>
        </div>
      )}
    </div>
  );
}

/* ── Main ─────────────────────────────────────────────────────────── */

export default function Equalizer({ setting, preset, send, model, defaultOpen = false }) {
  const { bandHz, fractionDigits, min, max } = setting.setting;
  const readOnly = !!setting.readOnly;
  const fd = fractionDigits || 0;

  const [open, setOpen] = useState(defaultOpen);
  const [bands, setBands] = useState(setting.value || []);
  const lastEdit = useRef(0);

  // Custom presets
  const [customPresets, setCustomPresets] = useState([]);
  const [showSave, setShowSave] = useState(false);

  // Load custom presets on mount and when model changes
  useEffect(() => {
    loadPresets(model || "").then(setCustomPresets);
  }, [model]);

  // Sync from device
  const incoming = (setting.value || []).join(",");
  useEffect(() => {
    if (Date.now() - lastEdit.current < SYNC_GRACE_MS) return;
    setBands(setting.value || []);
  }, [incoming]); // eslint-disable-line react-hooks/exhaustive-deps

  // Lock scroll when any EQ slider is focused
  useEffect(() => {
    const lock = (e) => {
      const target = e.target;
      if (target?.getAttribute?.("role") === "slider") {
        let el = target.parentElement;
        while (el && el !== document.body) {
          const style = getComputedStyle(el);
          if (style.overflow === "auto" || style.overflow === "scroll" ||
              style.overflowY === "auto" || style.overflowY === "scroll") {
            el.dataset.scrollLocked = el.style.overflow;
            el.style.overflow = "hidden";
          }
          el = el.parentElement;
        }
      }
    };
    const unlock = (e) => {
      const target = e.target;
      if (target?.getAttribute?.("role") === "slider") {
        let el = target.parentElement;
        while (el && el !== document.body) {
          if (el.dataset.scrollLocked !== undefined) {
            el.style.overflow = el.dataset.scrollLocked;
            delete el.dataset.scrollLocked;
          }
          el = el.parentElement;
        }
      }
    };
    document.addEventListener("focusin", lock);
    document.addEventListener("focusout", unlock);
    return () => {
      document.removeEventListener("focusin", lock);
      document.removeEventListener("focusout", unlock);
    };
  }, []);

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

  // Device presets
  const deviceOptions = preset?.setting?.options ?? null;

  // All preset names: device presets + custom presets
  const allPresetNames = useMemo(() => {
    const device = deviceOptions ?? Object.keys(PRESETS);
    const custom = customPresets.map((p) => p.name);
    return [...device, ...custom];
  }, [deviceOptions, customPresets]);

  // Which custom preset matches current bands (by value)
  const activeCustomId = useMemo(() => {
    const bandStr = bands.join(",");
    const match = customPresets.find((p) => p.bands === bandStr);
    return match?.id ?? null;
  }, [bands, customPresets]);

  const activePreset = deviceOptions
    ? (preset?.value ?? (activeCustomId ? customPresets.find((p) => p.id === activeCustomId)?.name : "") ?? "Custom")
    : (Object.keys(PRESETS).find(
        (name) =>
          resample(PRESETS[name] ?? [], bandHz.length)
            .map((v) => clamp(v * 10 ** fd, min, max))
            .join(",") === bands.join(","),
      ) ?? customPresets.find((p) => p.bands === bands.join(","))?.name ?? "Custom");

  // Check if bands differ from ALL presets (for save button)
  const isCustomBands = useMemo(() => {
    const bandStr = bands.join(",");
    if (customPresets.some((p) => p.bands === bandStr)) return false;
    if (deviceOptions) return true; // always show save for device presets
    return !Object.keys(PRESETS).some(
      (name) =>
        resample(PRESETS[name] ?? [], bandHz.length)
          .map((v) => clamp(v * 10 ** fd, min, max))
          .join(",") === bandStr,
    );
  }, [bands, customPresets, deviceOptions, bandHz.length, fd, min, max]);

  const applyPreset = (name) => {
    if (readOnly || !name) return;
    // Check custom presets first
    const custom = customPresets.find((p) => p.name === name);
    if (custom) {
      commit(custom.bands.split(",").map(Number));
      return;
    }
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

  const handleDeletePreset = async (id) => {
    await deletePreset(id);
    setCustomPresets((prev) => prev.filter((p) => p.id !== id));
  };

  const handleRenamePreset = async (id, newName) => {
    await renamePreset(id, newName);
    setCustomPresets((prev) => prev.map((p) => p.id === id ? { ...p, name: newName } : p));
  };

  const handleSaved = async () => {
    setShowSave(false);
    const updated = await loadPresets(model || "");
    setCustomPresets(updated);
  };

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

            {/* Save dialog */}
            {showSave && (
              <SaveDialog
                bands={bands}
                model={model || ""}
                onSaved={handleSaved}
                onCancel={() => setShowSave(false)}
              />
            )}

            {/* Preset pills */}
            <div className="grid grid-cols-2 gap-2">
              {allPresetNames.map((name) => {
                const isCustom = customPresets.some((p) => p.name === name);
                const customId = customPresets.find((p) => p.name === name)?.id;
                const active = name === activePreset;
                return (
                  <PresetPill
                    key={name}
                    name={name}
                    active={active}
                    isCustom={isCustom}
                    readOnly={readOnly}
                    onSelect={applyPreset}
                    onRename={(newName) => customId && handleRenamePreset(customId, newName)}
                    onDelete={() => customId && handleDeletePreset(customId)}
                  />
                );
              })}

              {/* Save button */}
              {!readOnly && isCustomBands && !showSave && (
                <button
                  type="button"
                  onClick={() => setShowSave(true)}
                  className="flex items-center justify-center gap-1.5 rounded-full bg-brand/10 px-3 py-2 text-[12px] font-medium text-brand transition hover:bg-brand/20"
                >
                  <Save className="h-3.5 w-3.5" /> Save as preset
                </button>
              )}

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
