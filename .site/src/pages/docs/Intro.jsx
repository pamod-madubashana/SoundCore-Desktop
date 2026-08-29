import { Link } from "react-router-dom";
import { DocTitle, H2, P, UL } from "../components/Prose";

const cards = [
  { to: "/docs/installation", title: "Installation", body: "Prerequisites, dependency setup, and running the app." },
  { to: "/docs/features", title: "Features", body: "What SoundCore-Desktop supports and how it helps." },
  { to: "/docs/usage", title: "Usage", body: "Launch the tray app, control profiles, restore settings." },
  { to: "/docs/architecture", title: "Architecture", body: "Project internals, modules, and folder structure." },
  { to: "/docs/troubleshooting", title: "Troubleshooting", body: "Common problems and recovery steps." },
  { to: "/docs/deployment", title: "Deployment", body: "Docs and static site publishing information." },
];

export default function Intro() {
  return (
    <article>
      <DocTitle
        eyebrow="docs / introduction"
        title="SoundCore-Desktop documentation"
        lead="SoundCore-Desktop is a lightweight cross-platform tray app that restores your Soundcore device settings automatically when your device connects."
      />

      <H2>What this project includes</H2>
      <UL
        items={[
          "Automatic Soundcore profile restore for gaming mode, ANC, and EQ",
          "Native desktop tray controls on Windows and Linux",
          "OpenSCQ30-based Bluetooth device handling",
          "A Tauri + React desktop app architecture",
        ]}
      />

      <H2>What you will find here</H2>
      <P>Each section below covers one part of installing, using, and contributing to the project.</P>
      <div className="mt-6 grid gap-3 sm:grid-cols-2">
        {cards.map((c) => (
          <Link
            key={c.to}
            to={c.to}
            className="rounded-lg border border-border bg-surface p-4 transition-colors hover:border-primary/35 hover:bg-surface-raised"
          >
            <p className="text-[14px] font-semibold tracking-tight">{c.title}</p>
            <p className="mt-1 text-[13px] leading-relaxed text-muted-foreground">{c.body}</p>
          </Link>
        ))}
      </div>
    </article>
  );
}
