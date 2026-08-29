import { Link } from "react-router-dom";
import { SiteHeader } from "../components/SiteHeader";
import { SiteFooter } from "../components/SiteFooter";

const GITHUB = "https://github.com/pamod-madubashana/SoundCore-Desktop";
const RELEASE = `${GITHUB}/releases/latest/download`;

const features = [
  {
    n: "01",
    title: "Automatic profile restore",
    body: "When your Soundcore device reconnects, your preferred settings are applied instantly — no mobile app in the loop.",
  },
  {
    n: "02",
    title: "Native tray controls",
    body: "Lives in the system tray on Windows and Linux. Left-click for the panel, right-click for the context menu.",
  },
  {
    n: "03",
    title: "Soundcore presets",
    body: "Gaming mode, ANC and ambient sound, equalizer curves, volume and profile presets — saved and reapplied.",
  },
  {
    n: "04",
    title: "Cross-platform build",
    body: "Tauri for native packaging, React + Vite for the tray UI, Rust for device handling and system integration.",
  },
  {
    n: "05",
    title: "OpenSCQ30 powered",
    body: "Device communication runs on the OpenSCQ30 library, handling the Soundcore Bluetooth protocol reliably.",
  },
  {
    n: "06",
    title: "Lightweight by design",
    body: "A small runtime footprint, no unnecessary UI, focused on one job: restoring your saved preferences.",
  },
];

const steps = [
  {
    n: "01",
    title: "Connect your device",
    body: "Pair or connect your Soundcore headphones over Bluetooth as usual.",
  },
  {
    n: "02",
    title: "Configure and save",
    body: "Open the tray window, set your mode, ANC state and EQ profile, then hit Save.",
  },
  {
    n: "03",
    title: "Reconnect and forget",
    body: "The app detects the connection event and reapplies the saved profile automatically.",
  },
];

export default function Home() {
  return (
    <div className="min-h-screen">
      <SiteHeader />

      {/* hero */}
      <section className="relative overflow-hidden border-b border-border grid-bg">
        <div className="pointer-events-none absolute -left-32 -top-24 size-[28rem] rounded-full bg-primary/10 blur-3xl" />
        <div className="pointer-events-none absolute -right-40 top-32 size-[24rem] rounded-full bg-primary/[0.06] blur-3xl" />

        <div className="relative mx-auto grid max-w-6xl items-center gap-14 px-6 py-20 lg:grid-cols-[1.05fr_0.95fr] lg:py-28">
          <div>
            <div className="inline-flex items-center gap-2 rounded-full border border-border bg-surface px-3 py-1">
              <span className="size-1.5 rounded-full bg-primary" />
              <span className="font-mono text-[11px] text-muted-foreground">
                Tauri · React · Rust · OpenSCQ30
              </span>
            </div>

            <h1 className="mt-6 max-w-[16ch] text-[2.75rem] font-semibold leading-[1.03] tracking-[-0.03em] text-balance sm:text-[3.5rem]">
              Your Soundcore settings, restored the second you connect.
            </h1>

            <p className="mt-6 max-w-[52ch] text-[15.5px] leading-[1.7] text-muted-foreground text-pretty">
              SoundCore-Desktop is a lightweight cross-platform tray app that reapplies your gaming
              mode, ANC and equalizer profile automatically — so you never open the mobile app again.
            </p>

            <div className="mt-9 flex flex-wrap items-center gap-3">
              <Link
                to="/downloads"
                className="rounded-md border border-primary/50 bg-primary/15 px-5 py-2.5 font-mono text-[13px] font-medium text-primary transition-colors hover:bg-primary/25"
              >
                Download for Windows
              </Link>
              <Link
                to="/docs/installation"
                className="rounded-md border border-border px-5 py-2.5 font-mono text-[13px] transition-colors hover:border-foreground/30"
              >
                Read the docs
              </Link>
            </div>

            <dl className="mt-10 flex flex-wrap gap-x-10 gap-y-4 border-t border-border pt-6">
              {[
                ["Windows 10/11", "supported"],
                ["Linux", "deb · rpm"],
                ["Tray-only", "no window clutter"],
              ].map(([value, label]) => (
                <div key={value}>
                  <dt className="font-mono text-[14px] text-foreground">{value}</dt>
                  <dd className="mt-0.5 font-mono text-[11px] text-faint">{label}</dd>
                </div>
              ))}
            </dl>
          </div>

          {/* app preview mock — matches actual UI */}
          <div className="relative">
            <div className="pointer-events-none absolute -inset-4 rounded-2xl bg-primary/10 blur-3xl" />
            <div className="relative rounded-xl border border-border bg-surface p-5 glow-panel">
              {/* header */}
              <div className="flex items-center justify-between border-b border-border pb-3">
                <div className="flex items-center gap-3">
                  <div className="grid size-10 place-items-center rounded-lg border border-border bg-background">
                    <svg className="size-5 text-muted-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                      <path d="M12 18.5a1.5 1.5 0 100-3 1.5 1.5 0 000 3z" />
                      <path d="M8 6a4 4 0 018 0v6a4 4 0 11-8 0V6z" />
                    </svg>
                  </div>
                  <div>
                    <p className="text-[13px] font-semibold">R50i NC</p>
                    <div className="flex items-center gap-2 font-mono text-[10px] text-faint">
                      <span className="flex items-center gap-1"><span className="size-1.5 rounded-full bg-primary" /> L 60%</span>
                      <span className="flex items-center gap-1"><span className="size-1.5 rounded-full bg-primary" /> R 60%</span>
                    </div>
                  </div>
                </div>
                <span className="flex items-center gap-1.5 font-mono text-[11px] text-primary">
                  <span className="size-1.5 rounded-full bg-primary" />
                  connected
                </span>
              </div>

              {/* sound mode */}
              <div className="flex gap-2 py-3">
                {["Noise Cancel", "Normal", "Transparency"].map((mode, i) => (
                  <div key={mode} className={`flex-1 rounded-lg border px-2 py-2 text-center text-[11px] font-medium ${
                    i === 2
                      ? "border-primary/50 bg-primary/15 text-primary"
                      : "border-border bg-background text-muted-foreground"
                  }`}>
                    {mode}
                  </div>
                ))}
              </div>

              {/* equalizer */}
              <div className="border-t border-border pt-3">
                <p className="font-mono text-[10px] uppercase tracking-wider text-faint mb-2">Equalizer</p>
                <div className="flex items-end gap-1.5 h-10">
                  {[45, 70, 58, 40, 52, 66, 48, 35].map((h, i) => (
                    <span
                      key={i}
                      className="flex-1 rounded-[2px] bg-primary/70"
                      style={{ height: `${h}%` }}
                    />
                  ))}
                </div>
                <div className="flex gap-1.5 mt-1">
                  {["100", "200", "400", "800", "1.6k", "3.2k", "6.4k", "12.8k"].map((f) => (
                    <span key={f} className="flex-1 text-center font-mono text-[8px] text-faint">{f}</span>
                  ))}
                </div>
              </div>

              {/* toggles */}
              <div className="space-y-1 border-t border-border pt-3 mt-3">
                {[
                  ["Gaming Mode", false],
                  ["Wind Noise Suppression", true],
                  ["Normal Mode In Cycle", true],
                ].map(([k, v]) => (
                  <div key={k} className="flex items-center justify-between rounded-md px-1 py-1.5">
                    <span className="font-mono text-[11px] text-muted-foreground">{k}</span>
                    <span className={`relative h-4 w-7 rounded-full transition-colors ${v ? "bg-primary" : "bg-white/10"}`}>
                      <span className={`absolute top-[2px] h-[12px] w-[12px] rounded-full bg-white shadow transition-all ${v ? "left-[16px]" : "left-[2px]"}`} />
                    </span>
                  </div>
                ))}
              </div>

              <p className="mt-3 border-t border-border pt-3 font-mono text-[10px] text-faint">
                <span className="text-primary">▸</span> profile reapplied on connect · 1s ago
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* features */}
      <section className="border-b border-border">
        <div className="mx-auto max-w-6xl px-6 py-20">
          <p className="eyebrow">Features</p>
          <h2 className="mt-2 max-w-[26ch] text-[1.9rem] font-semibold tracking-[-0.02em] text-balance">
            Built to make Soundcore devices behave on desktop.
          </h2>

          <div className="mt-10 grid gap-px overflow-hidden rounded-xl border border-border bg-border sm:grid-cols-2 lg:grid-cols-3">
            {features.map((f) => (
              <div
                key={f.n}
                className="bg-surface p-6 transition-colors hover:bg-surface-raised"
              >
                <p className="font-mono text-[11px] text-primary">{f.n}</p>
                <h3 className="mt-3 text-[15px] font-semibold tracking-tight">{f.title}</h3>
                <p className="mt-2 text-[13.5px] leading-[1.7] text-muted-foreground">{f.body}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* workflow */}
      <section className="border-b border-border">
        <div className="mx-auto grid max-w-6xl gap-12 px-6 py-20 lg:grid-cols-[0.95fr_1.05fr]">
          <div>
            <p className="eyebrow">Workflow</p>
            <h2 className="mt-2 max-w-[22ch] text-[1.9rem] font-semibold tracking-[-0.02em] text-balance">
              Configure once. It handles the rest.
            </h2>
            <ol className="mt-9 space-y-7">
              {steps.map((s) => (
                <li key={s.n} className="flex gap-4">
                  <span className="font-mono text-[13px] text-primary">{s.n}</span>
                  <div>
                    <h3 className="text-[15px] font-semibold tracking-tight">{s.title}</h3>
                    <p className="mt-1 max-w-[46ch] text-[13.5px] leading-[1.7] text-muted-foreground">
                      {s.body}
                    </p>
                  </div>
                </li>
              ))}
            </ol>
          </div>

          <div className="overflow-hidden rounded-xl border border-border bg-surface self-start">
            <div className="flex items-center gap-2 border-b border-border px-4 py-2.5">
              <span className="size-2.5 rounded-full bg-faint/50" />
              <span className="size-2.5 rounded-full bg-faint/50" />
              <span className="size-2.5 rounded-full bg-primary/70" />
              <span className="ml-2 font-mono text-[11px] text-faint">bash — SoundCore-Desktop</span>
            </div>
            <pre className="overflow-x-auto px-4 py-4 font-mono text-[12.5px] leading-[1.85]">
              <code>
                <span className="block text-faint"># clone and install</span>
                <span className="block">
                  <span className="text-faint">$ </span>git clone https://github.com/pamod-madubashana/SoundCore-Desktop.git
                </span>
                <span className="block">
                  <span className="text-faint">$ </span>cd SoundCore-Desktop
                </span>
                <span className="block">
                  <span className="text-faint">$ </span>npm install
                </span>
                <span className="block">
                  <span className="text-faint">$ </span>npm --prefix ui install
                </span>
                <span className="block text-faint"># run the tray app</span>
                <span className="block">
                  <span className="text-faint">$ </span>npm run tauri dev
                </span>
                <span className="block text-primary">✓ tray icon registered · watching for devices</span>
              </code>
            </pre>
          </div>
        </div>
      </section>

      {/* architecture */}
      <section className="border-b border-border">
        <div className="mx-auto max-w-6xl px-6 py-20">
          <p className="eyebrow">Architecture</p>
          <h2 className="mt-2 max-w-[28ch] text-[1.9rem] font-semibold tracking-[-0.02em] text-balance">
            A Rust core for devices, a React shell for the tray.
          </h2>

          <div className="mt-10 grid gap-4 md:grid-cols-2 lg:grid-cols-4">
            {[
              ["ui/", "React + Vite frontend for the tray UI and user interaction."],
              ["src-tauri/", "Rust backend with Tauri integration, system APIs and device handling."],
              ["docs/", "Docusaurus documentation site and markdown content."],
              [".site/", "This website — built with React + Tailwind CSS."],
            ].map(([path, body]) => (
              <div key={path} className="rounded-xl border border-border bg-surface p-5">
                <p className="font-mono text-[13px] text-primary">{path}</p>
                <p className="mt-2 text-[13px] leading-[1.7] text-muted-foreground">{body}</p>
              </div>
            ))}
          </div>

          <div className="mt-4">
            <Link
              to="/docs/architecture"
              className="inline-block font-mono text-[12.5px] text-muted-foreground underline-offset-4 transition-colors hover:text-primary hover:underline"
            >
              Full architecture reference →
            </Link>
          </div>
        </div>
      </section>

      {/* download */}
      <section>
        <div className="mx-auto max-w-6xl px-6 py-20">
          <p className="eyebrow">Downloads</p>
          <h2 className="mt-2 text-[1.9rem] font-semibold tracking-[-0.02em]">Get SoundCore-Desktop</h2>

          <div className="mt-8 grid gap-4 sm:grid-cols-2">
            <div className="rounded-xl border border-border bg-surface p-6 transition-colors hover:border-primary/30">
              <div className="flex items-center justify-between">
                <span className="font-mono text-[11.5px] uppercase tracking-wider text-muted-foreground">
                  Windows
                </span>
                <span className="font-mono text-[11px] text-faint">x64</span>
              </div>
              <div className="mt-4 flex flex-wrap gap-2.5">
                <a
                  href={`${RELEASE}/SoundCore-Desktop-x64-setup.exe`}
                  className="rounded-md border border-primary/50 bg-primary/10 px-3.5 py-2 font-mono text-[12px] text-primary transition-colors hover:bg-primary/20"
                >
                  x64-setup.exe
                </a>
                <a
                  href={`${RELEASE}/SoundCore-Desktop.exe`}
                  className="rounded-md border border-border px-3.5 py-2 font-mono text-[12px] transition-colors hover:border-foreground/30"
                >
                  portable .exe
                </a>
              </div>
            </div>

            <div className="rounded-xl border border-border bg-surface p-6 transition-colors hover:border-primary/30">
              <div className="flex items-center justify-between">
                <span className="font-mono text-[11.5px] uppercase tracking-wider text-muted-foreground">
                  Linux
                </span>
                <span className="font-mono text-[11px] text-faint">amd64 · x86_64</span>
              </div>
              <div className="mt-4 flex flex-wrap gap-2.5">
                <a
                  href={`${RELEASE}/SoundCore-Desktop-amd64.deb`}
                  className="rounded-md border border-border px-3.5 py-2 font-mono text-[12px] transition-colors hover:border-foreground/30"
                >
                  amd64.deb
                </a>
                <a
                  href={`${RELEASE}/SoundCore-Desktop-x86_64.rpm`}
                  className="rounded-md border border-border px-3.5 py-2 font-mono text-[12px] transition-colors hover:border-foreground/30"
                >
                  x86_64.rpm
                </a>
              </div>
            </div>
          </div>

          <p className="mt-5 font-mono text-[11px] text-faint">
            Builds are published on the GitHub releases page · Windows portable build is a standalone EXE
          </p>
        </div>
      </section>

      <SiteFooter />
    </div>
  );
}
