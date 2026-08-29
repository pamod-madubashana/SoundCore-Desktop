import { DocTitle, H2, P, UL, Terminal } from "../components/Prose";

export default function Installation() {
  return (
    <article>
      <DocTitle
        eyebrow="docs / installation"
        title="Installation"
        lead="Set up the repository, run the tray app in development, and produce native builds."
      />

      <H2>Prerequisites</H2>
      <UL
        items={["Node.js 18+", "Rust toolchain", "npm installed", "Windows 10/11 or Linux"]}
      />

      <H2>Install the repository</H2>
      <Terminal
        lines={[
          "git clone https://github.com/pamod-madubashana/SoundCore-Desktop.git",
          "cd SoundCore-Desktop",
          "npm install",
          "npm --prefix ui install",
        ]}
      />

      <H2>Run the desktop app in development</H2>
      <Terminal lines={["npm run tauri dev"]} />
      <P>
        This starts the Tauri development environment and opens the app in the system tray.
      </P>

      <H2>Build for production</H2>
      <Terminal lines={["npm run tauri build"]} />
      <P>
        The release output is produced by Tauri and can be packaged for Windows or Linux.
      </P>

      <H2>Run the docs locally</H2>
      <Terminal lines={["cd docs", "npm install", "npm run start"]} />
      <P>Then open the local development URL shown in the terminal.</P>
    </article>
  );
}
