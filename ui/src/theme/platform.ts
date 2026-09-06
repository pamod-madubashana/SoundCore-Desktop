const invoke: (cmd: string, args?: Record<string, unknown>) => Promise<any> =
  (window as any).__TAURI__?.core?.invoke ?? (async () => {});

/**
 * Detect the running platform via the Rust backend.
 * Returns "windows", "linux", "macos", or "browser" (if Tauri unavailable).
 */
export async function detectPlatform(): Promise<string> {
  try {
    return await invoke("get_platform");
  } catch {
    return typeof navigator !== "undefined" ? "browser" : "unknown";
  }
}

/**
 * Read the Windows system accent color from the registry.
 * Returns a hex string like "#0078D4", or null on non-Windows / failure.
 */
export async function getAccentColor(): Promise<string | null> {
  try {
    return await invoke("get_windows_accent");
  } catch {
    return null;
  }
}

/**
 * Combined call — fetches platform + accent in one round-trip.
 * Returns { os: string, accentColor: string | null, accentLight2: string | null }
 */
export async function getPlatformInfo(): Promise<{
  os: string;
  accentColor: string | null;
  accentLight2: string | null;
}> {
  try {
    const info = await invoke("get_platform_info");
    return { os: info.os, accentColor: info.accent_color, accentLight2: info.accent_light2 };
  } catch {
    return {
      os: typeof navigator !== "undefined" ? "browser" : "unknown",
      accentColor: null,
      accentLight2: null,
    };
  }
}

/**
 * Convert a "#RRGGBB" hex color to an OKLCH string.
 * Falls back to a default if conversion fails.
 */
export function hexToOklch(hex: unknown, fallback: string = "oklch(0.78 0.16 195)"): string {
  if (!hex || typeof hex !== "string") return fallback;

  const m = hex.match(/^#?([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i);
  if (!m) return fallback;

  const r = parseInt(m[1], 16) / 255;
  const g = parseInt(m[2], 16) / 255;
  const b = parseInt(m[3], 16) / 255;

  // sRGB linearization
  const lin: (c: number) => number = (c) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
  const rl = lin(r),
    gl = lin(g),
    bl = lin(b);

  // Linear sRGB → OKLab (Björn Ottosson's matrix)
  const l_ = 0.4122214708 * rl + 0.5363325363 * gl + 0.0514459929 * bl;
  const m_ = 0.2119034982 * rl + 0.6806995451 * gl + 0.1073969566 * bl;
  const s_ = 0.0883024619 * rl + 0.2817188376 * gl + 0.6299787005 * bl;

  const l_c = Math.sign(l_) * Math.pow(Math.abs(l_), 1 / 3);
  const m_c = Math.sign(m_) * Math.pow(Math.abs(m_), 1 / 3);
  const s_c = Math.sign(s_) * Math.pow(Math.abs(s_), 1 / 3);

  const L = 0.2104542553 * l_c + 0.7936177850 * m_c - 0.0040720468 * s_c;
  const a = 1.9779984951 * l_c - 2.4285922050 * m_c + 0.4505937099 * s_c;
  const bb = 0.0259040371 * l_c + 0.7827717662 * m_c - 0.8086757660 * s_c;

  // OKLab → OKLCH
  const C = Math.sqrt(a * a + bb * bb);
  let H = (Math.atan2(bb, a) * 180) / Math.PI;
  if (H < 0) H += 360;

  return `oklch(${L.toFixed(4)} ${C.toFixed(4)} ${H.toFixed(1)})`;
}
