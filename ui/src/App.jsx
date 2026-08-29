import { useEffect, useRef, useState } from "react";
import {
  Volume2, Waves, Ear, Loader2, X,
} from "lucide-react";
const invoke = window.__TAURI__?.core?.invoke ?? (async () => {});

// Device-type illustration picked from the name (no reliable per-model photo source exists).
// Uses the device color to tint the SVG template.
function DeviceArt({ name = "", url, color }) {
  if (url) return <img src={url} alt="" className="h-full w-full object-cover" />;
  const n = name.toLowerCase();
  const cat = /motion|boom|flare|select|rave/.test(n)
    ? "speaker"
    : /space|vortex|life tune|life q|(^|\s)q\d/.test(n)
      ? "overear"
      : "earbuds";
  
  const fill = "var(--brand)";
  
  if (cat === "earbuds") {
    const lineColor = "#888888";
    return (
      <svg width="100%" height="100%" viewBox="0 0 479 479" xmlns="http://www.w3.org/2000/svg" style={{ color: fill, shapeRendering: 'geometricPrecision' }}>
        <g>
          <path fill="currentColor" d="M 287.00 140.00 L 271.00 147.00 L 262.00 153.00 L 253.00 161.00 L 242.00 176.00 L 235.00 194.00 L 233.00 203.00 L 233.00 211.00 L 232.00 212.00 L 233.00 231.00 L 236.00 242.00 L 245.00 259.00 L 258.00 273.00 L 272.00 283.00 L 286.00 290.00 L 299.00 294.00 L 303.00 297.00 L 311.00 326.00 L 314.00 333.00 L 314.00 336.00 L 324.00 366.00 L 324.00 369.00 L 327.00 376.00 L 327.00 379.00 L 330.00 386.00 L 330.00 389.00 L 343.00 429.00 L 343.00 432.00 L 346.00 441.00 L 352.00 452.00 L 359.00 459.00 L 365.00 463.00 L 377.00 467.00 L 391.00 467.00 L 402.00 464.00 L 413.00 458.00 L 418.00 453.00 L 424.00 443.00 L 427.00 432.00 L 427.00 415.00 L 425.00 405.00 L 422.00 396.00 L 422.00 392.00 L 418.00 379.00 L 418.00 375.00 L 414.00 362.00 L 414.00 358.00 L 403.00 313.00 L 415.00 309.00 L 431.00 300.00 L 444.00 288.00 L 450.00 280.00 L 457.00 265.00 L 460.00 250.00 L 460.00 238.00 L 457.00 224.00 L 448.00 207.00 L 437.00 195.00 L 427.00 188.00 L 417.00 183.00 L 402.00 179.00 L 396.00 171.00 L 386.00 161.00 L 377.00 154.00 L 363.00 146.00 L 344.00 139.00 L 328.00 137.00 L 327.00 136.00 L 306.00 136.00 Z"/>
          <path fill="currentColor" d="M 189.00 25.00 L 175.00 22.00 L 149.00 22.00 L 127.00 27.00 L 111.00 34.00 L 100.00 41.00 L 89.00 50.00 L 76.00 65.00 L 61.00 69.00 L 51.00 74.00 L 39.00 83.00 L 29.00 95.00 L 22.00 109.00 L 19.00 122.00 L 19.00 139.00 L 21.00 148.00 L 27.00 163.00 L 34.00 173.00 L 43.00 182.00 L 56.00 191.00 L 75.00 199.00 L 75.00 204.00 L 53.00 294.00 L 51.00 310.00 L 52.00 311.00 L 53.00 324.00 L 58.00 335.00 L 65.00 343.00 L 77.00 350.00 L 89.00 353.00 L 105.00 352.00 L 118.00 346.00 L 127.00 337.00 L 134.00 323.00 L 134.00 320.00 L 138.00 310.00 L 138.00 307.00 L 141.00 300.00 L 141.00 297.00 L 144.00 290.00 L 144.00 287.00 L 151.00 267.00 L 151.00 264.00 L 154.00 257.00 L 154.00 254.00 L 157.00 247.00 L 157.00 244.00 L 160.00 237.00 L 160.00 234.00 L 163.00 227.00 L 163.00 224.00 L 167.00 214.00 L 167.00 211.00 L 176.00 182.00 L 179.00 180.00 L 192.00 176.00 L 208.00 168.00 L 218.00 161.00 L 235.00 143.00 L 243.00 127.00 L 246.00 115.00 L 246.00 91.00 L 243.00 77.00 L 238.00 64.00 L 226.00 47.00 L 217.00 39.00 L 208.00 33.00 Z"/>
        </g>
        <g fill={lineColor} transform="translate(0 480) scale(0.1 -0.1)">
          <path d="M1476 4570 c-246 -39 -495 -174 -639 -347 -50 -59 -70 -76 -102 -84 -427 -106 -648 -516 -488 -903 45 -109 125 -207 240 -294 51 -39 189 -110 232 -118 17 -4 34 -10 37 -15 2 -5 -48 -230 -112 -501 -114 -479 -118 -496 -118 -608 0 -97 4 -124 23 -173 43 -108 119 -185 228 -228 49 -20 77 -24 173 -24 107 0 119 2 175 29 73 36 144 106 179 175 15 28 123 372 241 764 118 391 218 718 223 726 4 7 41 26 82 41 511 187 724 577 569 1043 -103 311 -396 511 -764 522 -66 2 -147 0 -179 -5z m304 -31 c172 -28 323 -104 446 -224 68 -68 89 -96 127 -175 70 -145 82 -199 82 -375 -1 -137 -3 -156 -27 -223 -38 -105 -83 -175 -163 -257 -77 -79 -154 -134 -260 -187 -75 -38 -193 -83 -199 -77 -2 2 4 31 14 64 10 33 69 227 131 430 121 400 134 469 109 588 -52 251 -308 430 -557 389 -104 -17 -218 -78 -288 -154 -94 -102 -109 -145 -231 -671 l-109 -469 -42 47 c-55 60 -95 132 -129 230 -25 71 -28 94 -28 205 0 137 14 207 64 316 122 270 394 474 716 538 84 17 256 20 344 5z m-105 -84 c82 -22 145 -59 210 -123 104 -104 160 -267 136 -398 -17 -91 -706 -2370 -735 -2430 -50 -104 -135 -172 -250 -201 -58 -14 -79 -15 -144 -4 -181 28 -296 129 -332 292 -26 116 -13 189 159 919 89 377 211 897 272 1155 87 372 117 485 145 542 96 199 324 304 539 248z m-965 -420 c-58 -116 -83 -219 -83 -350 0 -184 57 -339 169 -463 45 -50 50 -60 43 -87 -4 -16 -10 -42 -13 -57 -4 -16 -10 -28 -14 -28 -12 0 -98 112 -141 182 -161 265 -180 588 -48 785 31 46 88 93 112 93 8 0 -2 -29 -25 -75z m-94 23 c-9 -13 -30 -49 -46 -80 -128 -245 -43 -660 186 -907 l54 -59 -19 -81 c-10 -45 -21 -85 -25 -88 -11 -11 -148 46 -221 92 -84 52 -198 168 -242 246 -55 95 -78 189 -78 314 0 89 5 124 24 183 52 158 160 283 324 373 63 35 64 35 43 7z"/>
          <path d="M1467 4390 c-107 -27 -215 -116 -260 -212 -22 -49 -579 -2349 -593 -2451 -8 -58 18 -157 56 -212 70 -101 168 -150 291 -143 118 7 205 60 264 163 33 56 716 2374 722 2451 13 150 -78 304 -223 375 -49 24 -78 32 -140 35 -44 2 -96 -1 -117 -6z m255 -58 c118 -58 188 -166 196 -302 4 -74 2 -84 -317 -1165 -217 -733 -330 -1102 -347 -1128 -29 -44 -89 -90 -150 -113 -64 -24 -183 -16 -249 18 -96 50 -154 136 -162 241 -4 55 16 146 172 797 97 404 214 895 261 1090 47 195 94 373 104 396 42 85 121 154 218 191 23 8 69 12 122 11 74 -3 95 -8 152 -36z m-1025 -2568 c42 -95 159 -172 276 -181 81 -7 165 16 231 64 28 20 46 28 43 20 -3 -9 -11 -33 -16 -54 -34 -125 -156 -216 -291 -216 -197 0 -333 180 -290 385 12 56 15 59 23 37 6 -13 17 -38 24 -55z"/>
          <path d="M1640 4030 c-6 -12 -10 -23 -8 -25 2 -2 28 -6 60 -10 50 -6 57 -5 67 13 14 26 6 31 -58 39 -43 5 -50 3 -61 -17z"/>
          <path d="M1440 3952 c-22 -10 -42 -32 -54 -56 -17 -36 -18 -43 -5 -80 16 -46 67 -86 108 -86 61 0 106 53 164 192 l15 36 -30 4 c-24 4 -32 0 -40 -18 l-10 -22 -36 24 c-41 29 -65 30 -112 6z m90 -62 c11 -11 20 -29 20 -40 0 -11 -9 -29 -20 -40 -11 -11 -29 -20 -40 -20 -11 0 -29 9 -40 20 -11 11 -20 29 -20 40 0 11 9 29 20 40 11 11 29 20 40 20 11 0 29 -9 40 -20z"/>
          <path d="M800 1545 c-6 -8 -10 -20 -7 -27 4 -8 6 -7 6 4 1 9 9 22 19 27 9 6 12 11 6 11 -6 0 -17 -7 -24 -15z"/>
          <path d="M898 1523 c7 -3 16 -2 19 1 4 3 -2 6 -13 5 -11 0 -14 -3 -6 -6z"/>
          <path d="M940 1516 c0 -2 11 -6 25 -8 13 -3 22 -1 19 3 -5 9 -44 13 -44 5z"/>
          <path d="M840 1480 c8 -5 20 -10 25 -10 6 0 3 5 -5 10 -8 5 -19 10 -25 10 -5 0 -3 -5 5 -10z"/>
          <path d="M888 1463 c7 -3 16 -2 19 1 4 3 -2 6 -13 5 -11 0 -14 -3 -6 -6z"/>
          <path d="M948 1453 c7 -3 16 -2 19 1 4 3 -2 6 -13 5 -11 0 -14 -3 -6 -6z"/>
          <path d="M3052 3430 c-226 -32 -429 -141 -553 -297 -67 -85 -108 -166 -141 -278 -19 -68 -23 -103 -23 -230 0 -139 2 -156 28 -230 37 -103 85 -182 164 -265 103 -110 226 -187 407 -257 50 -19 92 -36 93 -37 1 -1 99 -323 218 -716 118 -393 229 -743 245 -777 106 -216 379 -284 603 -149 118 71 181 199 181 366 -1 86 -12 143 -117 588 -64 272 -117 500 -117 507 0 7 24 21 53 31 238 78 429 274 487 500 24 95 26 232 4 318 -58 228 -258 425 -494 485 -62 15 -72 22 -107 67 -142 178 -372 314 -610 360 -97 18 -244 25 -321 14z m360 -49 c300 -78 532 -258 652 -506 60 -122 79 -206 79 -335 0 -188 -47 -306 -190 -479 -1 -2 -5 -2 -8 0 -2 3 -54 217 -115 475 -74 312 -120 489 -139 529 -85 181 -252 286 -451 286 -144 0 -247 -43 -353 -148 -94 -93 -157 -270 -141 -396 4 -29 56 -216 116 -417 61 -201 120 -400 133 -442 l22 -77 -71 26 c-158 56 -316 157 -413 263 -178 196 -224 450 -132 737 83 259 305 440 612 499 99 19 298 11 399 -15z m-82 -61 c132 -25 275 -134 333 -253 20 -41 105 -383 305 -1230 276 -1169 277 -1172 276 -1282 0 -90 -4 -120 -23 -167 -99 -251 -458 -319 -644 -121 -60 63 -81 114 -177 433 -50 168 -122 406 -160 530 -38 124 -102 338 -144 475 -41 138 -130 430 -196 650 -134 442 -144 494 -116 605 65 254 297 407 546 360z m758 -364 c163 -86 221 -406 124 -686 -38 -109 -122 -251 -196 -333 -38 -41 -34 -44 -56 41 l-11 43 51 57 c65 72 133 208 155 308 36 161 8 364 -73 517 -35 67 -35 67 -27 67 3 0 18 -6 33 -14z m156 -58 c37 -20 95 -66 147 -118 69 -68 94 -101 124 -163 85 -179 81 -383 -13 -562 -69 -130 -216 -260 -370 -326 -103 -44 -104 -43 -126 56 l-19 84 50 55 c101 110 183 263 224 416 19 68 23 111 23 215 0 116 -3 139 -27 209 -15 43 -41 98 -58 122 -16 24 -27 44 -23 44 3 0 34 -15 68 -32z"/>
          <path d="M3160 3250 c-114 -24 -217 -107 -272 -218 -25 -50 -32 -80 -36 -142 -5 -85 -11 -59 143 -580 126 -428 246 -831 335 -1135 179 -608 222 -750 237 -779 28 -54 92 -113 154 -141 56 -25 70 -27 148 -23 71 3 95 9 139 33 106 57 163 147 170 266 4 71 -4 112 -113 565 -383 1599 -456 1898 -473 1936 -68 152 -266 252 -432 218z m231 -57 c70 -33 127 -85 165 -149 26 -45 30 -61 344 -1374 233 -973 226 -930 174 -1037 -99 -205 -400 -226 -531 -37 -25 35 -36 71 -343 1114 -127 432 -249 844 -270 915 -55 181 -62 226 -49 299 25 138 109 238 242 287 78 29 184 22 268 -18z m759 -2568 c21 -109 -5 -197 -79 -278 -59 -63 -127 -92 -221 -91 -61 0 -85 5 -126 27 -65 33 -128 104 -154 173 -29 78 -27 94 8 62 70 -66 200 -95 308 -69 97 24 215 133 231 213 9 45 20 33 33 -37z"/>
          <path d="M3368 2978 c-41 -18 -48 -24 -48 -49 0 -16 2 -29 4 -29 2 0 29 11 61 25 42 19 55 30 51 41 -3 9 -6 20 -6 25 0 12 -7 11 -62 -13z"/>
          <path d="M3342 2868 c-6 -6 -12 -22 -12 -34 0 -20 -4 -22 -33 -17 -67 11 -127 -42 -127 -112 0 -54 30 -100 75 -115 44 -15 74 -8 114 26 38 31 44 65 34 174 -8 87 -22 107 -51 78z m-16 -133 c20 -30 12 -70 -16 -85 -27 -14 -57 -4 -76 25 -13 19 -13 28 -3 47 24 46 69 52 95 13z"/>
          <path d="M3991 394 c11 -14 10 -19 -10 -35 -13 -11 -19 -19 -14 -19 6 0 17 8 27 19 17 18 13 51 -6 51 -6 0 -4 -7 3 -16z"/>
          <path d="M3843 373 c9 -2 23 -2 30 0 6 3 -1 5 -18 5 -16 0 -22 -2 -12 -5z"/>
          <path d="M3804 355 c-9 -24 15 -45 50 -44 l31 2 -35 6 c-28 5 -35 10 -34 29 1 26 -4 28 -12 7z"/>
          <path d="M3898 323 c7 -3 16 -2 19 1 4 3 -2 6 -13 5 -11 0 -14 -3 -6 -6z"/>
        </g>
      </svg>
    );
  }
  
  if (cat === "speaker") {
    return (
      <svg width="34" height="34" viewBox="0 0 56 56" fill="none" xmlns="http://www.w3.org/2000/svg">
        <rect x="14" y="6" width="28" height="44" rx="8" fill={fill} />
        <circle cx="28" cy="32" r="10" fill="rgba(255,255,255,0.1)" stroke="rgba(255,255,255,0.2)" strokeWidth="1.5" />
        <circle cx="28" cy="32" r="4" fill="rgba(255,255,255,0.15)" />
        <circle cx="28" cy="14" r="3" fill="rgba(255,255,255,0.2)" />
      </svg>
    );
  }
  
  if (cat === "overear") {
    return (
      <svg width="34" height="34" viewBox="0 0 56 56" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M12 28v-4a16 16 0 0 1 32 0v4" stroke={fill} strokeWidth="3" strokeLinecap="round" fill="none" />
        <rect x="8" y="26" width="10" height="16" rx="5" fill={fill} />
        <rect x="38" y="26" width="10" height="16" rx="5" fill={fill} />
      </svg>
    );
  }
  
  return (
    <svg width="34" height="34" viewBox="0 0 56 56" fill="none" xmlns="http://www.w3.org/2000/svg">
      <rect x="14" y="10" width="28" height="36" rx="6" fill={fill} />
      <circle cx="28" cy="30" r="10" fill="rgba(255,255,255,0.1)" stroke="rgba(255,255,255,0.2)" strokeWidth="1" />
    </svg>
  );
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
        <Footer />
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
  const send = (id, raw) => invoke("set_setting", { mac: d.mac_address, id, raw });
  return (
    <>
      <Header d={d} s={s} />
      <div className="flex-1 overflow-y-auto px-4 py-4 space-y-4">
        {s.ambientSoundMode && <SoundMode s={s} send={send} />}
        {s.volumeAdjustments && <Equalizer setting={s.volumeAdjustments} send={send} />}
        <QuickToggles s={s} send={send} />
      </div>
    </>
  );
}

const APP_VERSION = "1.2.0";

function Footer() {
  return (
    <div className="px-3 py-2 flex items-center justify-between text-[10px] text-muted-foreground/60">
      <span>v{APP_VERSION}</span>
      <button onClick={() => invoke("open_url", { url: "https://github.com/pamod-madubashana/SoundCore-Desktop" })}
        className="hover:text-muted-foreground transition cursor-pointer">GitHub</button>
    </div>
  );
}

function batteryPct(setting) {
  if (!setting) return null;
  // Prefer translatedValue (e.g. "80%") from OpenSCQ30
  const tv = setting.translatedValue ?? "";
  const tm = String(tv).match(/(\d+)\s*%/);
  if (tm) return Math.min(100, Number(tm[1]));
  // Fallback: parse raw value "4/5" or "8/10"
  const v = setting.value ?? "";
  const m = String(v).match(/(\d+)\s*\/\s*(\d+)/);
  if (m) {
    const level = Number(m[1]);
    const max = Number(m[2]);
    if (max <= 1) return 100;
    return Math.min(100, Math.round((level / max) * 100));
  }
  const n = Number(v);
  return Number.isFinite(n) ? Math.min(100, n) : null;
}

function BatteryIcon({ level }) {
  // level: 0-100
  const pct = Math.max(0, Math.min(100, level ?? 0));
  const color = pct > 50 ? "text-success" : pct > 20 ? "text-yellow-500" : "text-red-500";
  return (
    <svg width="18" height="11" viewBox="0 0 18 11" fill="none" className={color}>
      <rect x="0.5" y="0.5" width="15" height="10" rx="2" stroke="currentColor" strokeWidth="1" opacity="0.4" />
      <rect x="16" y="3" width="2" height="5" rx="0.5" fill="currentColor" opacity="0.4" />
      <rect x="1.5" y="1.5" width={Math.max(0, 13 * pct / 100)} height="8" rx="1" fill="currentColor" />
    </svg>
  );
}

function Header({ d, s }) {
  const batteries = [
    ["L", batteryPct(s.batteryLevelLeft)],
    ["R", batteryPct(s.batteryLevelRight)],
    ["", batteryPct(s.batteryLevel)],
  ].filter(([, v]) => v != null);

  return (
    <header className="p-4 flex items-center gap-4 border-b border-white/[0.05]">
      <div className={"relative overflow-hidden flex-shrink-0 flex items-center justify-center text-brand " + (d.image ? "h-28 w-28" : "h-14 w-14 rounded-xl bg-surface-elevated ring-1 ring-white/5")}>
        <DeviceArt name={d.name} url={d.image} color={d.color} />
      </div>
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-1.5">
          <h1 className="text-[15px] font-semibold tracking-tight truncate">{d.name?.replace(/^soundcore\s+/i, '') || d.model}</h1>
          <span className={"h-1.5 w-1.5 rounded-full " + (d.connected ? "bg-success brand-glow" : "bg-white/20")} />
        </div>
        {batteries.length > 0 && (
          <div className="flex items-center gap-3 mt-2">
            {batteries.map(([label, v]) => (
              <div key={label || "b"} className="flex items-center gap-1.5 text-[13px]">
                <BatteryIcon level={v} />
                <span className="text-foreground font-medium">{v}%</span>
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
    { kw: "noise", label: "Noise Cancel", Icon: Ear },
    { kw: "normal", label: "Normal", Icon: Volume2 },
    { kw: "transparen", label: "Transparency", Icon: Waves },
  ].map((m) => ({ ...m, opt: pickOption(opts, m.kw) })).filter((m) => m.opt);

  const manual = s.manualNoiseCanceling; // i32Range, optional
  const showStrength = manual && /noise/i.test(value || "");

  return (
    <div className="rounded-xl bg-surface p-3 ring-1 ring-white/[0.04]">
      <div className="grid grid-cols-3 gap-1.5 p-1 rounded-lg bg-black/30">
        {modes.map(({ opt, label, Icon }) => {
          const activeMode = opt === value;
          return (
            <button key={opt} onClick={() => send("ambientSoundMode", opt)}
              className={"relative flex flex-col items-center justify-center gap-1.5 py-2.5 rounded-md text-[11px] font-medium leading-tight transition-all " +
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
  const toggles = Object.values(s).filter((x) => x.type === "toggle")
    .sort((a, b) => (/gam/i.test(a.id) ? -1 : /gam/i.test(b.id) ? 1 : 0));
  if (toggles.length === 0) return null;
  return (
    <div className="rounded-xl bg-surface ring-1 ring-white/[0.04] divide-y divide-white/[0.04]">
      {toggles.map((t) => <ToggleRow key={t.id} t={t} send={send} />)}
    </div>
  );
}

function ToggleRow({ t, send }) {
  const on = !!t.value;
  return (
    <button onClick={() => send(t.id, String(!on))}
      className="w-full flex items-center gap-3 px-3 py-2.5 hover:bg-white/[0.02] transition text-left">
      <div className="flex-1 min-w-0 text-[13px] font-medium">{pretty(t.id)}</div>
      <span className={"relative h-[18px] w-8 rounded-full transition-colors flex-shrink-0 " + (on ? "bg-brand" : "bg-white/10")}>
        <span className={"absolute top-[2px] h-[14px] w-[14px] rounded-full bg-white shadow transition-all " + (on ? "left-[16px]" : "left-[2px]")} />
      </span>
    </button>
  );
}
