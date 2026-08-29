import { DocTitle, H2, P, UL } from "../components/Prose";

export default function Features() {
  return (
    <article>
      <DocTitle
        eyebrow="docs / features"
        title="Features"
        lead="SoundCore-Desktop is designed to make Soundcore device management seamless on desktop."
      />

      <H2>Automatic profile restore</H2>
      <P>
        When your Soundcore device reconnects, the app automatically applies your preferred profile
        settings without requiring the mobile app.
      </P>

      <H2>Tray app controls</H2>
      <UL
        items={[
          "Runs in the system tray on Windows and Linux",
          "Quick access to apply, save, and restore profiles",
          "Minimal UI for fast workflow",
        ]}
      />

      <H2>Support for Soundcore presets</H2>
      <UL
        items={["Gaming mode", "ANC / ambient sound mode", "Equalizer settings", "Volume and profile presets"]}
      />

      <H2>Cross-platform architecture</H2>
      <UL
        items={[
          "Built with Tauri for native desktop packaging",
          "React + Vite frontend for the tray UI",
          "Rust backend for device handling and system integration",
        ]}
      />

      <H2>OpenSCQ30-powered device support</H2>
      <P>
        The project uses the OpenSCQ30 library to handle the Soundcore Bluetooth protocol and restore
        device settings reliably.
      </P>

      <H2>Lightweight and efficient</H2>
      <UL
        items={[
          "Small runtime footprint",
          "No unnecessary UI distractions",
          "Focused on restoring saved device preferences automatically",
        ]}
      />
    </article>
  );
}
