import { useEffect, useRef, useState } from "react";
import { detectPlatform, getPlatformInfo } from "./platform";

/**
 * Resolves the platform theme and accent color before React renders.
 * Sets classes on <html> so the correct CSS theme is active from the start.
 *
 * Usage in main.tsx:
 *   const ready = useThemeSetup();
 *   if (!ready) return null;
 *   createRoot(root).render(<App />);
 */
export function useThemeSetup(): boolean {
  const [ready, setReady] = useState(false);
  const initRef = useRef(false);

  useEffect(() => {
    if (initRef.current) return;
    initRef.current = true;

    (async () => {
      const html = document.documentElement;

      // 1. Detect platform
      const platform = await detectPlatform();
      const isWindows = platform === "windows";
      html.classList.add(isWindows ? "theme-windows" : "theme-default");

      // 2. Determine dark/light mode
      if (isWindows) {
        const prefersDark = window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? true;
        html.dataset.mode = prefersDark ? "dark" : "light";

        // Listen for system theme changes
        const mq = window.matchMedia?.("(prefers-color-scheme: dark)");
        if (mq) {
          const handler = (e: MediaQueryListEvent): void => { html.dataset.mode = e.matches ? "dark" : "light"; };
          mq.addEventListener("change", handler);
        }
      } else {
        html.dataset.mode = "dark";
      }

      // 3. Accent color handled by CSS defaults in theme-windows.css

      setReady(true);
    })();
  }, []);

  return ready;
}
