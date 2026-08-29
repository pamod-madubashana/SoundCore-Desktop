import { useState } from "react";

/* ── Mock device data ──────────────────────────────────────────────── */
const MOCK_DEVICE = {
  name: "Space A40",
  model: "A3936",
  connected: true,
  batteryL: 80,
  batteryR: 80,
  soundMode: "noise_cancel",
  strength: 3,
  eq: [6, 4, 2, -1, 1, 3, 5, 2],
  gamingMode: false,
  windNoise: true,
  normalInCycle: true,
};

const EQ_HZ = [100, 200, 400, 800, 1600, 3200, 6400, 12800];
const EQ_MIN = -6;
const EQ_MAX = 6;
const BAND_LABEL = (hz) => (hz >= 1000 ? hz / 1000 + "k" : String(hz));

const SOUND_MODES = [
  { id: "noise_cancel", label: "Noise Cancel", kw: "noise" },
  { id: "normal", label: "Normal", kw: "normal" },
  { id: "transparency", label: "Transparency", kw: "transparen" },
];

const TOGGLES = [
  { key: "gamingMode", label: "Gaming Mode" },
  { key: "windNoise", label: "Wind Noise Suppression" },
  { key: "normalInCycle", label: "Normal Mode In Cycle" },
];

/* ── SVG icons (inline, no lucide dep) ────────────────────────────── */
function IconEar(props) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <path d="M6 8.5a6.5 6.5 0 1 1 13 0c0 6-6 6-6 10.5" />
      <path d="M15 8.5a2.5 2.5 0 0 0-5 0v1a2 2 0 0 0 4 0" />
    </svg>
  );
}
function IconVolume(props) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
      <path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
      <path d="M19.07 4.93a10 10 0 0 1 0 14.14" />
    </svg>
  );
}
function IconWaves(props) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <path d="M2 12h2a4 4 0 0 0 4-4V6" />
      <path d="M22 12h-2a4 4 0 0 1-4-4V6" />
      <path d="M2 6v6a4 4 0 0 0 4 4" />
      <path d="M22 6v6a4 4 0 0 1-4 4" />
    </svg>
  );
}
function IconX(props) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <path d="M18 6 6 18" /><path d="m6 6 12 12" />
    </svg>
  );
}
const ICONS = { noise_cancel: IconEar, normal: IconVolume, transparency: IconWaves };

/* ── DeviceArt (earbuds SVG from real app) ─────────────────────────── */
function DeviceArt({ name = "" }) {
  const fill = "var(--demo-brand)";
  const lineColor = "#888888";
  return (
    <svg width="100%" height="100%" viewBox="0 0 479 479" xmlns="http://www.w3.org/2000/svg" style={{ color: fill, shapeRendering: "geometricPrecision" }}>
      <g>
        <path fill="currentColor" d="M 287.00 140.00 L 271.00 147.00 L 262.00 153.00 L 253.00 161.00 L 242.00 176.00 L 235.00 194.00 L 233.00 203.00 L 233.00 211.00 L 232.00 212.00 L 233.00 231.00 L 236.00 242.00 L 245.00 259.00 L 258.00 273.00 L 272.00 283.00 L 286.00 290.00 L 299.00 294.00 L 303.00 297.00 L 311.00 326.00 L 314.00 333.00 L 314.00 336.00 L 324.00 366.00 L 324.00 369.00 L 327.00 376.00 L 327.00 379.00 L 330.00 386.00 L 330.00 389.00 L 343.00 429.00 L 343.00 432.00 L 346.00 441.00 L 352.00 452.00 L 359.00 459.00 L 365.00 463.00 L 377.00 467.00 L 391.00 467.00 L 402.00 464.00 L 413.00 458.00 L 418.00 453.00 L 424.00 443.00 L 427.00 432.00 L 427.00 415.00 L 425.00 405.00 L 422.00 396.00 L 422.00 392.00 L 418.00 379.00 L 418.00 375.00 L 414.00 362.00 L 414.00 358.00 L 403.00 313.00 L 415.00 309.00 L 431.00 300.00 L 444.00 288.00 L 450.00 280.00 L 457.00 265.00 L 460.00 250.00 L 460.00 238.00 L 457.00 224.00 L 448.00 207.00 L 437.00 195.00 L 427.00 188.00 L 417.00 183.00 L 402.00 179.00 L 396.00 171.00 L 386.00 161.00 L 377.00 154.00 L 363.00 146.00 L 344.00 139.00 L 328.00 137.00 L 327.00 136.00 L 306.00 136.00 Z"/>
        <path fill="currentColor" d="M 189.00 25.00 L 175.00 22.00 L 149.00 22.00 L 127.00 27.00 L 111.00 34.00 L 100.00 41.00 L 89.00 50.00 L 76.00 65.00 L 61.00 69.00 L 51.00 74.00 L 39.00 83.00 L 29.00 95.00 L 22.00 109.00 L 19.00 122.00 L 19.00 139.00 L 21.00 148.00 L 27.00 163.00 L 34.00 173.00 L 43.00 182.00 L 56.00 191.00 L 75.00 199.00 L 75.00 204.00 L 53.00 294.00 L 51.00 310.00 L 52.00 311.00 L 53.00 324.00 L 58.00 335.00 L 65.00 343.00 L 77.00 350.00 L 89.00 353.00 L 105.00 352.00 L 118.00 346.00 L 127.00 337.00 L 134.00 323.00 L 134.00 320.00 L 138.00 310.00 L 138.00 307.00 L 141.00 300.00 L 141.00 297.00 L 144.00 290.00 L 144.00 287.00 L 151.00 267.00 L 151.00 264.00 L 154.00 257.00 L 154.00 254.00 L 157.00 247.00 L 157.00 244.00 L 160.00 237.00 L 160.00 234.00 L 163.00 227.00 L 163.00 224.00 L 167.00 214.00 L 167.00 211.00 L 176.00 182.00 L 179.00 180.00 L 192.00 176.00 L 208.00 168.00 L 218.00 161.00 L 235.00 143.00 L 243.00 127.00 L 246.00 115.00 L 246.00 91.00 L 243.00 77.00 L 238.00 64.00 L 226.00 47.00 L 217.00 39.00 L 208.00 33.00 Z"/>
      </g>
      <g fill={lineColor} transform="translate(0 480) scale(0.1 -0.1)">
        <path d="M1476 4570 c-246 -39 -495 -174 -639 -347 -50 -59 -70 -76 -102 -84 -427 -106 -648 -516 -488 -903 45 -109 125 -207 240 -294 51 -39 189 -110 232 -118 17 -4 34 -10 37 -15 2 -5 -48 -230 -112 -501 -114 -479 -118 -496 -118 -608 0 -97 4 -124 23 -173 43 -108 119 -185 228 -228 49 -20 77 -24 173 -24 107 0 119 2 175 29 73 36 144 106 179 175 15 28 123 372 241 764 118 391 218 718 223 726 4 7 41 26 82 41 511 187 724 577 569 1043 -103 311 -396 511 -764 522 -66 2 -147 0 -179 -5z m304 -31 c172 -28 323 -104 446 -224 68 -68 89 -96 127 -175 70 -145 82 -199 82 -375 -1 -137 -3 -156 -27 -223 -38 -105 -83 -175 -163 -257 -77 -79 -154 -134 -260 -187 -75 -38 -193 -83 -199 -77 -2 2 4 31 14 64 10 33 69 227 131 430 121 400 134 469 109 588 -52 251 -308 430 -557 389 -104 -17 -218 -78 -288 -154 -94 -102 -109 -145 -231 -671 l-109 -469 -42 47 c-55 60 -95 132 -129 230 -25 71 -28 94 -28 205 0 137 14 207 64 316 122 270 394 474 716 538 84 17 256 20 344 5z"/>
        <path d="M1467 4390 c-107 -27 -215 -116 -260 -212 -22 -49 -579 -2349 -593 -2451 -8 -58 18 -157 56 -212 70 -101 168 -150 291 -143 118 7 205 60 264 163 33 56 716 2374 722 2451 13 150 -78 304 -223 375 -49 24 -78 32 -140 35 -44 2 -96 -1 -117 -6z m255 -58 c118 -58 188 -166 196 -302 4 -74 2 -84 -317 -1165 -217 -733 -330 -1102 -347 -1128 -29 -44 -89 -90 -150 -113 -64 -24 -183 -16 -249 18 -96 50 -154 136 -162 241 -4 55 16 146 172 797 97 404 214 895 261 1090 47 195 94 373 104 396 42 85 121 154 218 191 23 8 69 12 122 11 74 -3 95 -8 152 -36z"/>
        <path d="M1640 4030 c-6 -12 -10 -23 -8 -25 2 -2 28 -6 60 -10 50 -6 57 -5 67 13 14 26 6 31 -58 39 -43 5 -50 3 -61 -17z"/>
        <path d="M1440 3952 c-22 -10 -42 -32 -54 -56 -17 -36 -18 -43 -5 -80 16 -46 67 -86 108 -86 61 0 106 53 164 192 l15 36 -30 4 c-24 4 -32 0 -40 -18 l-10 -22 -36 24 c-41 29 -65 30 -112 6z"/>
        <path d="M3052 3430 c-226 -32 -429 -141 -553 -297 -67 -85 -108 -166 -141 -278 -19 -68 -23 -103 -23 -230 0 -139 2 -156 28 -230 37 -103 85 -182 164 -265 103 -110 226 -187 407 -257 50 -19 92 -36 93 -37 1 -1 99 -323 218 -716 118 -393 229 -743 245 -777 106 -216 379 -284 603 -149 118 71 181 199 181 366 -1 86 -12 143 -117 588 -64 272 -117 500 -117 507 0 7 24 21 53 31 238 78 429 274 487 500 24 95 26 232 4 318 -58 228 -258 425 -494 485 -62 15 -72 22 -107 67 -142 178 -372 314 -610 360 -97 18 -244 25 -321 14z"/>
        <path d="M3160 3250 c-114 -24 -217 -107 -272 -218 -25 -50 -32 -80 -36 -142 -5 -85 -11 -59 143 -580 126 -428 246 -831 335 -1135 179 -608 222 -750 237 -779 28 -54 92 -113 154 -141 56 -25 70 -27 148 -23 71 3 95 9 139 33 106 57 163 147 170 266 4 71 -4 112 -113 565 -383 1599 -456 1898 -473 1936 -68 152 -266 252 -432 218z"/>
        <path d="M3342 2868 c-6 -6 -12 -22 -12 -34 0 -20 -4 -22 -33 -17 -67 11 -127 -42 -127 -112 0 -54 30 -100 75 -115 44 -15 74 -8 114 26 38 31 44 65 34 174 -8 87 -22 107 -51 78z"/>
      </g>
    </svg>
  );
}

/* ── Battery icon ──────────────────────────────────────────────────── */
function BatteryIcon({ level }) {
  const pct = Math.max(0, Math.min(100, level ?? 0));
  const color = pct > 50 ? "text-emerald-400" : pct > 20 ? "text-yellow-400" : "text-red-400";
  return (
    <svg width="18" height="11" viewBox="0 0 18 11" fill="none" className={color}>
      <rect x="0.5" y="0.5" width="15" height="10" rx="2" stroke="currentColor" strokeWidth="1" opacity="0.4" />
      <rect x="16" y="3" width="2" height="5" rx="0.5" fill="currentColor" opacity="0.4" />
      <rect x="1.5" y="1.5" width={Math.max(0, (13 * pct) / 100)} height="8" rx="1" fill="currentColor" />
    </svg>
  );
}

/* ── LiveDemo: self-contained interactive app preview ──────────────── */
export default function LiveDemo() {
  const [device, setDevice] = useState({ ...MOCK_DEVICE });

  const setSoundMode = (mode) => setDevice((d) => ({ ...d, soundMode: mode }));
  const setStrength = (v) => setDevice((d) => ({ ...d, strength: v }));
  const setEqBand = (i, v) =>
    setDevice((d) => ({ ...d, eq: d.eq.map((x, idx) => (idx === i ? v : x)) }));
  const toggle = (key) => setDevice((d) => ({ ...d, [key]: !d[key] }));

  return (
    <div className="demo-shell relative rounded-xl border border-white/[0.06] bg-[#16162a] overflow-hidden shadow-2xl">
      {/* glow */}
      <div className="pointer-events-none absolute -inset-8 rounded-2xl bg-[var(--demo-brand)]/10 blur-3xl" />

      <div className="relative">
        {/* ── Header ─────────────────────────────────────── */}
          <header className="flex items-center gap-4 border-b border-white/[0.06] p-4">
          <div className="relative flex h-28 w-28 flex-shrink-0 items-center justify-center overflow-hidden rounded-xl bg-white/[0.03]">
            <img src="/devices/a3936_black_com_device.png" alt="Space A40" className="h-full w-full object-contain p-1" />
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-1.5">
              <h1 className="truncate text-[15px] font-semibold tracking-tight">{device.name}</h1>
              <span className="h-1.5 w-1.5 rounded-full bg-emerald-400 shadow-[0_0_8px_var(--demo-brand)]" />
            </div>
            <div className="mt-2 flex items-center gap-3">
              <div className="flex items-center gap-1.5 text-[13px]">
                <BatteryIcon level={device.batteryL} />
                <span className="font-medium">{device.batteryL}%</span>
              </div>
              <div className="flex items-center gap-1.5 text-[13px]">
                <BatteryIcon level={device.batteryR} />
                <span className="font-medium">{device.batteryR}%</span>
              </div>
            </div>
          </div>
          <button className="-mr-1 -mt-1 self-start rounded-md p-1 text-white/30 transition hover:bg-white/5 hover:text-white/60">
            <IconX className="h-4 w-4" />
          </button>
        </header>

        {/* ── Scrollable body ────────────────────────────── */}
        <div className="space-y-4 overflow-y-auto px-4 py-4">
          {/* Sound mode */}
          <div className="rounded-xl bg-white/[0.03] p-3 ring-1 ring-white/[0.04]">
            <div className="grid grid-cols-3 gap-1.5 rounded-lg bg-black/30 p-1">
              {SOUND_MODES.map((m) => {
                const active = device.soundMode === m.id;
                const Icon = ICONS[m.id];
                return (
                  <button
                    key={m.id}
                    onClick={() => setSoundMode(m.id)}
                    className={`relative flex flex-col items-center justify-center gap-1.5 rounded-md py-2.5 text-[11px] font-medium leading-tight transition-all ${
                      active
                        ? "bg-[var(--demo-brand)] text-[var(--demo-brand-fg)] shadow-[0_0_24px_-6px_var(--demo-brand-glow)]"
                        : "text-white/40 hover:bg-white/5 hover:text-white/60"
                    }`}
                  >
                    <Icon className="h-4 w-4" />
                    {m.label}
                  </button>
                );
              })}
            </div>
            {/* Strength slider (only when noise cancel) */}
            {device.soundMode === "noise_cancel" && (
              <div className="mt-3 px-1">
                <div className="mb-1.5 flex items-center justify-between text-[11px]">
                  <span className="text-white/40">Strength</span>
                  <span className="font-medium tabular-nums">{device.strength}</span>
                </div>
                <input
                  type="range"
                  min={1}
                  max={5}
                  value={device.strength}
                  onChange={(e) => setStrength(Number(e.target.value))}
                  className="demo-range w-full"
                />
              </div>
            )}
          </div>

          {/* Equalizer */}
          <div className="rounded-xl bg-white/[0.03] p-3 ring-1 ring-white/[0.04]">
            <h2 className="mb-2.5 text-[11px] font-medium uppercase tracking-wider text-white/40">
              Equalizer
            </h2>
            <div className="demo-eq-track flex items-end justify-between gap-1 px-1" style={{ height: 112 }}>
              {EQ_HZ.map((hz, i) => (
                <div key={hz} className="flex flex-1 flex-col items-center gap-1">
                  <input
                    type="range"
                    min={EQ_MIN}
                    max={EQ_MAX}
                    step={1}
                    value={device.eq[i]}
                    onChange={(e) => setEqBand(i, Number(e.target.value))}
                    className="demo-eq-slider"
                  />
                  <span className="tabular-nums text-[8.5px] text-white/40">{BAND_LABEL(hz)}</span>
                  <span className="tabular-nums text-[8.5px] text-[var(--demo-brand)]">
                    {device.eq[i] > 0 ? "+" : ""}
                    {device.eq[i]}
                  </span>
                </div>
              ))}
            </div>
          </div>

          {/* Toggles */}
          <div className="divide-y divide-white/[0.04] rounded-xl bg-white/[0.03] ring-1 ring-white/[0.04]">
            {TOGGLES.map((t) => (
              <button
                key={t.key}
                onClick={() => toggle(t.key)}
                className="flex w-full items-center gap-3 px-3 py-2.5 text-left transition hover:bg-white/[0.02]"
              >
                <div className="min-w-0 flex-1 text-[13px] font-medium">{t.label}</div>
                <span
                  className={`relative flex-shrink-0 rounded-full transition-colors ${
                    device[t.key] ? "bg-[var(--demo-brand)]" : "bg-white/10"
                  }`}
                  style={{ height: 18, width: 32 }}
                >
                  <span
                    className={`absolute top-[2px] h-[14px] w-[14px] rounded-full bg-white shadow transition-all ${
                      device[t.key] ? "left-[16px]" : "left-[2px]"
                    }`}
                  />
                </span>
              </button>
            ))}
          </div>

          {/* Status bar */}
          <div className="border-t border-white/[0.06] pt-3 text-[10px] text-white/30">
            <span className="text-[var(--demo-brand)]">&#9656;</span> profile reapplied on connect &middot; just now
          </div>
        </div>
      </div>
    </div>
  );
}
