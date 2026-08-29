import { SiteHeader } from "../components/SiteHeader";
import { SiteFooter } from "../components/SiteFooter";

const GITHUB = "https://github.com/pamod-madubashana/SoundCore-Desktop";
const RELEASE = `${GITHUB}/releases/latest/download`;

const platforms = [
  {
    name: "Windows",
    meta: "Windows 10 / 11 · x64",
    builds: [
      {
        file: "SoundCore-Desktop-x64-setup.exe",
        label: "Installer",
        primary: true,
      },
      {
        file: "SoundCore-Desktop.exe",
        label: "Portable standalone EXE",
        primary: false,
      },
    ],
  },
  {
    name: "Linux",
    meta: "Common distributions · amd64 / x86_64",
    builds: [
      { file: "SoundCore-Desktop-amd64.deb", label: "Debian / Ubuntu", primary: true },
      { file: "SoundCore-Desktop-x86_64.rpm", label: "Fedora / RHEL / openSUSE", primary: false },
    ],
  },
];

export default function Downloads() {
  return (
    <div className="min-h-screen">
      <SiteHeader />

      <section className="border-b border-border grid-bg">
        <div className="mx-auto max-w-6xl px-6 py-16">
          <p className="eyebrow">Downloads</p>
          <h1 className="mt-3 max-w-[20ch] text-[2.25rem] font-semibold leading-[1.05] tracking-[-0.03em] text-balance">
            Latest SoundCore-Desktop builds
          </h1>
          <p className="mt-4 max-w-[58ch] text-[15px] leading-relaxed text-muted-foreground">
            Every build is published on the GitHub releases page. Pick the package for your platform
            and launch it once — the app registers itself in the system tray.
          </p>
        </div>
      </section>

      <section className="mx-auto max-w-6xl px-6 py-14">
        <div className="grid gap-5 md:grid-cols-2">
          {platforms.map((p) => (
            <div key={p.name} className="rounded-xl border border-border bg-surface p-6">
              <div className="flex items-baseline justify-between border-b border-border pb-4">
                <h2 className="text-[17px] font-semibold tracking-tight">{p.name}</h2>
                <span className="font-mono text-[11px] text-faint">{p.meta}</span>
              </div>
              <ul className="mt-4 space-y-3">
                {p.builds.map((b) => (
                  <li
                    key={b.file}
                    className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-border bg-background px-4 py-3"
                  >
                    <div className="min-w-0">
                      <p className="truncate font-mono text-[12.5px] text-foreground">{b.file}</p>
                      <p className="mt-0.5 font-mono text-[11px] text-faint">{b.label}</p>
                    </div>
                    <a
                      href={`${RELEASE}/${b.file}`}
                      className={
                        b.primary
                          ? "shrink-0 rounded-md border border-primary/50 bg-primary/10 px-3.5 py-1.5 font-mono text-[12px] text-primary transition-colors hover:bg-primary/20"
                          : "shrink-0 rounded-md border border-border px-3.5 py-1.5 font-mono text-[12px] transition-colors hover:border-foreground/30"
                      }
                    >
                      Download
                    </a>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        <div className="mt-8 rounded-xl border border-primary/25 bg-primary/[0.06] p-6">
          <h2 className="text-[15px] font-semibold tracking-tight">Notes</h2>
          <ul className="mt-3 space-y-2 text-[13.5px] leading-relaxed text-muted-foreground">
            <li className="flex gap-3">
              <span className="mt-2 size-1 shrink-0 rounded-full bg-primary" />
              The Windows portable build is a standalone EXE — no installation required.
            </li>
            <li className="flex gap-3">
              <span className="mt-2 size-1 shrink-0 rounded-full bg-primary" />
              The DEB and RPM packages support common Linux distributions.
            </li>
            <li className="flex gap-3">
              <span className="mt-2 size-1 shrink-0 rounded-full bg-primary" />
              Prefer building from source? Follow the{" "}
              <a href={GITHUB} className="text-primary underline-offset-4 hover:underline">
                repository
              </a>{" "}
              instructions.
            </li>
          </ul>
        </div>
      </section>

      <SiteFooter />
    </div>
  );
}
