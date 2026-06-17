import { useEffect, useRef, useState } from "react";
import {
  Battery, Settings, Power, Volume2, Waves, Ear, Sparkles, Loader2, X, Rocket,
} from "lucide-react";
const invoke = window.__TAURI__?.core?.invoke ?? (async () => {});

// Device-type illustration picked from the name (no reliable per-model photo source exists).
function DeviceArt({ name = "", url }) {
  if (url) return <img src={url} alt="" className="h-full w-full object-cover" />;
  const n = name.toLowerCase();
  const cat = /motion|boom|flare|select|rave/.test(n)
    ? "speaker"
    : /space|vortex|life tune|life q|(^|\s)q\d/.test(n)
      ? "overear"
      : "earbuds";
  const common = { width: 34, height: 34, viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: 1.6, strokeLinecap: "round", strokeLinejoin: "round" };
  if (cat === "speaker")
    return (<svg {...common}><rect x="6" y="2.5" width="12" height="19" rx="2.5" /><circle cx="12" cy="15" r="3.2" /><circle cx="12" cy="6.5" r="1" /></svg>);
  if (cat === "overear")
    return (<svg {...common}><path d="M5 13v-1a7 7 0 0 1 14 0v1" /><rect x="3.5" y="12.5" width="3.5" height="6.5" rx="1.6" /><rect x="17" y="12.5" width="3.5" height="6.5" rx="1.6" /></svg>);
  return (<svg {...common}><path d="M9 3.5C7 3.5 6 6 6 8.5S7.2 13 8.6 13c.9 0 1.4-.6 1.4-1.6V6.5C10 4.7 9.8 3.5 9 3.5Z" /><path d="M15 3.5c2 0 3 2.5 3 5s-1.2 4.5-2.6 4.5c-.9 0-1.4-.6-1.4-1.6V6.5c0-1.8.2-3 1-3Z" /><path d="M8.7 12.5 8 20M15.3 12.5 16 20" /></svg>);
}

const BAND_LABEL = (hz) => (hz >= 1000 ? hz / 1000 + "k" : String(hz));
const pretty = (id) => id.replace(/([A-Z])/g, " $1").replace(/^./, (c) => c.toUpperCase()).trim();

export default function App() {
  const [devices, setDevices] = useState([]);
  const interacting = useRef(false);

  useEffect(() => {
    const tick = async () => {
      if (interacting.current) return;
      try { setDevices(await invoke("get_states")); } catch { /* ignore */ }
    };
    tick();
    const t = setInterval(tick, 900);
    const down = () => (interacting.current = true);
    const up = () => setTimeout(() => (interacting.current = false), 250);
    window.addEventListener("pointerdown", down);
    window.addEventListener("pointerup", up);
    return () => { clearInterval(t); window.removeEventListener("pointerdown", down); window.removeEventListener("pointerup", up); };
  }, []);

  // Only surface a device once it's actually connected; otherwise keep searching for
  // ANY supported Soundcore device rather than pinning to a remembered (offline) one.
  const active = devices.find((d) => d.connected) || null;

  return (
    <main className="h-screen w-screen flex items-stretch justify-stretch">
      <section className="popup-window w-full h-full rounded-2xl overflow-hidden flex flex-col">
        {active ? <Device d={active} /> : <Searching />}
      </section>
    </main>
  );
}

function Searching() {
  return (
    <div className="flex-1 flex flex-col items-center justify-center gap-3 text-muted-foreground">
      <Loader2 className="h-7 w-7 animate-spin text-brand" />
      <p className="text-[13px]">Searching for a Soundcore device…</p>
      <p className="text-[11px] opacity-70">Connect it via Bluetooth and it'll appear here.</p>
    </div>
  );
}

function settingsMap(d) {
  const m = {};
  (d.categories || []).forEach((c) => c.settings.forEach((s) => (m[s.id] = s)));
  return m;
}

function Device({ d }) {
  const s = settingsMap(d);
  const [view, setView] = useState("main");
  const send = (id, raw) => invoke("set_setting", { mac: d.mac_address, id, raw });
  return (
    <>
      <Header d={d} s={s} />
      <div className="flex-1 overflow-y-auto px-4 py-4 space-y-4">
        {view === "settings" ? (
          <SettingsPanel d={d} />
        ) : (
          <>
            {s.ambientSoundMode && <SoundMode s={s} send={send} />}
            {s.volumeAdjustments && <Equalizer setting={s.volumeAdjustments} send={send} />}
            <QuickToggles s={s} send={send} />
          </>
        )}
      </div>
      <Footer mac={d.mac_address} view={view} setView={setView} />
    </>
  );
}

function batteryPct(setting) {
  if (!setting) return null;
  const v = setting.value ?? setting.translatedValue ?? "";
  const m = String(v).match(/(\d+)\s*\/\s*(\d+)/);
  if (m) return Math.round((Number(m[1]) / Number(m[2])) * 100);
  const n = Number(v);
  return Number.isFinite(n) ? n : null;
}

function Header({ d, s }) {
  const batteries = [
    ["L", batteryPct(s.batteryLevelLeft)],
    ["R", batteryPct(s.batteryLevelRight)],
    ["", batteryPct(s.batteryLevel)],
  ].filter(([, v]) => v != null);

  return (
    <header className="p-4 flex items-center gap-3 border-b border-white/[0.05]">
      <div className="relative h-14 w-14 rounded-xl bg-surface-elevated overflow-hidden flex-shrink-0 ring-1 ring-white/5 flex items-center justify-center text-brand">
        <DeviceArt name={d.name} url={d.image} />
      </div>
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-1.5">
          <h1 className="text-[15px] font-semibold tracking-tight truncate">{d.name || d.model}</h1>
          <span className={"h-1.5 w-1.5 rounded-full " + (d.connected ? "bg-success brand-glow" : "bg-white/20")} />
        </div>
        <p className="text-[11px] text-muted-foreground mt-0.5">{d.connected ? "Connected" : d.message || "Waiting…"}</p>
        {batteries.length > 0 && (
          <div className="flex items-center gap-3 mt-2">
            {batteries.map(([label, v]) => (
              <div key={label || "b"} className="flex items-center gap-1 text-[11px] text-muted-foreground">
                <Battery className="h-3 w-3 text-success" />
                <span className="text-foreground/90 font-medium">{v}%</span>
                {label && <span className="opacity-60">{label}</span>}
              </div>
            ))}
          </div>
        )}
      </div>
      <button onClick={() => invoke("hide_window")} title="Hide"
        className="self-start -mt-1 -mr-1 p-1 rounded-md text-muted-foreground hover:text-foreground hover:bg-white/5 transition">
        <X className="h-4 w-4" />
      </button>
    </header>
  );
}

function pickOption(options, kw) {
  return options.find((o) => o.toLowerCase().includes(kw));
}

function SoundMode({ s, send }) {
  const setting = s.ambientSoundMode;
  const opts = setting.setting.options;
  const value = setting.value;
  const modes = [
    { kw: "noise", label: "Noise\nCancel", Icon: Ear },
    { kw: "normal", label: "Normal", Icon: Volume2 },
    { kw: "transparen", label: "Trans-\nparency", Icon: Waves },
  ].map((m) => ({ ...m, opt: pickOption(opts, m.kw) })).filter((m) => m.opt);

  const manual = s.manualNoiseCanceling; // i32Range, optional
  const showStrength = manual && /noise/i.test(value || "");

  return (
    <div className="rounded-xl bg-surface p-3 ring-1 ring-white/[0.04]">
      <div className="flex items-center justify-between mb-2.5">
        <h2 className="text-[11px] font-medium uppercase tracking-wider text-muted-foreground">Sound Mode</h2>
        <Sparkles className="h-3 w-3 text-brand" />
      </div>
      <div className="grid grid-cols-3 gap-1.5 p-1 rounded-lg bg-black/30">
        {modes.map(({ opt, label, Icon }) => {
          const activeMode = opt === value;
          return (
            <button key={opt} onClick={() => send("ambientSoundMode", opt)}
              className={"relative flex flex-col items-center justify-center gap-1.5 py-2.5 rounded-md text-[11px] font-medium leading-tight whitespace-pre-line transition-all " +
                (activeMode ? "bg-brand text-brand-foreground brand-glow" : "text-muted-foreground hover:text-foreground hover:bg-white/5")}>
              <Icon className="h-4 w-4" />
              {label}
            </button>
          );
        })}
      </div>
      {showStrength && <Strength setting={manual} send={send} />}
    </div>
  );
}

function Strength({ setting, send }) {
  const { start, end } = setting.setting;
  const [v, setV] = useState(setting.value);
  const lastEdit = useRef(0);
  useEffect(() => {
    if (Date.now() - lastEdit.current < 2500) return; // keep a fresh local edit
    setV(setting.value);
  }, [setting.value]);
  return (
    <div className="mt-3 px-1">
      <div className="flex items-center justify-between text-[11px] mb-1.5">
        <span className="text-muted-foreground">Strength</span>
        <span className="font-medium tabular-nums">{v}</span>
      </div>
      <input type="range" min={start} max={end} value={v} className="w-full accent-brand h-1"
        onChange={(e) => { lastEdit.current = Date.now(); setV(Number(e.target.value)); }}
        onPointerUp={() => { lastEdit.current = Date.now(); send("manualNoiseCanceling", String(v)); }} />
    </div>
  );
}

function Equalizer({ setting, send }) {
  const { bandHz, fractionDigits, min, max } = setting.setting;
  const fd = fractionDigits || 0;
  const [bands, setBands] = useState(setting.value || []);
  const lastEdit = useRef(0);
  // Sync from the device only when the *content* changes AND we didn't just edit, so a
  // poll tick can't clobber the band you're dragging.
  const incoming = (setting.value || []).join(",");
  useEffect(() => {
    if (Date.now() - lastEdit.current < 2500) return;
    setBands(setting.value || []);
  }, [incoming]);

  const setBand = (i, v) => {
    lastEdit.current = Date.now();
    setBands((b) => b.map((x, idx) => (idx === i ? v : x)));
  };
  const commit = () => {
    lastEdit.current = Date.now();
    setBands((b) => { send("volumeAdjustments", b.join(",")); return b; });
  };

  return (
    <div className="rounded-xl bg-surface p-3 ring-1 ring-white/[0.04]">
      <h2 className="text-[11px] font-medium uppercase tracking-wider text-muted-foreground mb-2.5">Equalizer</h2>
      <div className="flex items-end justify-between gap-1 h-28 px-1 eq-track">
        {bandHz.map((hz, i) => (
          <div key={hz} className="flex flex-col items-center gap-1 flex-1">
            <input type="range" min={min} max={max} step={1} value={bands[i] ?? 0}
              style={{ writingMode: "vertical-lr", direction: "rtl", width: 14, height: 80, accentColor: "var(--brand)" }}
              onChange={(e) => setBand(i, Number(e.target.value))}
              onPointerUp={commit} />
            <span className="text-[8.5px] text-muted-foreground tabular-nums">{BAND_LABEL(hz)}</span>
            <span className="text-[8.5px] text-brand tabular-nums">{((bands[i] ?? 0) / 10 ** fd).toFixed(fd)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function QuickToggles({ s, send }) {
  const toggles = Object.values(s).filter((x) => x.type === "toggle");
  if (toggles.length === 0) return null;
  return (
    <div className="rounded-xl bg-surface ring-1 ring-white/[0.04] divide-y divide-white/[0.04]">
      {toggles.map((t) => <ToggleRow key={t.id} t={t} send={send} />)}
    </div>
  );
}

function ToggleRow({ t, send }) {
  const on = !!t.value;
  const Icon = /gam/i.test(t.id) ? Sparkles : /wear|detect/i.test(t.id) ? Ear : Volume2;
  return (
    <button onClick={() => send(t.id, String(!on))}
      className="w-full flex items-center gap-3 px-3 py-2.5 hover:bg-white/[0.02] transition text-left">
      <div className="h-7 w-7 rounded-md bg-black/30 flex items-center justify-center">
        <Icon className="h-3.5 w-3.5 text-brand" />
      </div>
      <div className="flex-1 min-w-0 text-[13px] font-medium">{pretty(t.id)}</div>
      <span className={"relative h-[18px] w-8 rounded-full transition-colors flex-shrink-0 " + (on ? "bg-brand" : "bg-white/10")}>
        <span className={"absolute top-[2px] h-[14px] w-[14px] rounded-full bg-white shadow transition-all " + (on ? "left-[16px]" : "left-[2px]")} />
      </span>
    </button>
  );
}

function Footer({ mac, view, setView }) {
  const inSettings = view === "settings";
  return (
    <footer className="px-3 py-2.5 border-t border-white/[0.05] flex items-center justify-between bg-black/20">
      <button onClick={() => invoke("apply_now", { mac })}
        className="flex items-center gap-1.5 text-[11.5px] text-muted-foreground hover:text-foreground transition px-2 py-1 rounded-md hover:bg-white/5">
        <Rocket className="h-3.5 w-3.5" /> Re-apply
      </button>
      <button onClick={() => setView(inSettings ? "main" : "settings")}
        className={"flex items-center gap-1.5 text-[11.5px] transition px-2 py-1 rounded-md hover:bg-white/5 " +
          (inSettings ? "text-brand" : "text-muted-foreground hover:text-foreground")}>
        <Settings className="h-3.5 w-3.5" /> {inSettings ? "Back" : "Settings"}
      </button>
      <button onClick={() => invoke("quit_app")}
        className="flex items-center gap-1.5 text-[11.5px] text-destructive/90 hover:text-destructive transition px-2 py-1 rounded-md hover:bg-destructive/10">
        <Power className="h-3.5 w-3.5" /> Quit
      </button>
    </footer>
  );
}

function InfoRow({ k, v }) {
  return (
    <div className="flex justify-between gap-3">
      <span className="text-muted-foreground">{k}</span>
      <span className="text-foreground/90 truncate">{v}</span>
    </div>
  );
}

function SettingsPanel({ d }) {
  const [autostart, setAutostart] = useState(null);
  useEffect(() => { invoke("get_config").then((c) => setAutostart(!!c.autostart)).catch(() => {}); }, []);
  const toggleAutostart = async () => {
    try {
      const c = await invoke("get_config");
      c.autostart = !c.autostart;
      await invoke("save_config", { newConfig: c });
      setAutostart(c.autostart);
    } catch { /* ignore */ }
  };
  return (
    <div className="space-y-4">
      <h2 className="text-[11px] font-medium uppercase tracking-wider text-muted-foreground">Settings</h2>
      <div className="rounded-xl bg-surface ring-1 ring-white/[0.04]">
        <div className="flex items-center gap-3 px-3 py-2.5">
          <div className="h-7 w-7 rounded-md bg-black/30 flex items-center justify-center">
            <Rocket className="h-3.5 w-3.5 text-brand" />
          </div>
          <div className="flex-1 min-w-0">
            <div className="text-[13px] font-medium">Run at startup</div>
            <div className="text-[10.5px] text-muted-foreground">Launch automatically when you log in</div>
          </div>
          <button onClick={toggleAutostart} disabled={autostart === null}
            className={"relative h-[18px] w-8 rounded-full transition-colors flex-shrink-0 " + (autostart ? "bg-brand" : "bg-white/10")}>
            <span className={"absolute top-[2px] h-[14px] w-[14px] rounded-full bg-white shadow transition-all " + (autostart ? "left-[16px]" : "left-[2px]")} />
          </button>
        </div>
      </div>
      <div className="rounded-xl bg-surface p-3 ring-1 ring-white/[0.04] text-[11.5px] space-y-1.5">
        <InfoRow k="Device" v={d.name} />
        <InfoRow k="Model" v={d.model} />
        <InfoRow k="MAC" v={d.mac_address} />
        <InfoRow k="Status" v={d.connected ? "Connected" : "Disconnected"} />
      </div>
    </div>
  );
}
