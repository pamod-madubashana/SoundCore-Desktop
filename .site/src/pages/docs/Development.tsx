import { DocTitle, H2, H3, P, UL, OL, Terminal, Code } from "../../components/Prose";

export default function Development() {
  return (
    <article>
      <DocTitle
        eyebrow="docs / development"
        title="Development"
        lead="Notes on how to contribute to the SoundCore-Desktop project."
      />

      <H2>Project structure</H2>
      <UL
        items={[
          <>
            <Code>ui/</Code> — React + Vite frontend code for the tray UI.
          </>,
          <>
            <Code>src-tauri/</Code> — Rust backend code and Tauri integration.
          </>,
          <>
            <Code>docs/</Code> — documentation site powered by Docusaurus.
          </>,
          <>
            <Code>.site/</Code> — this website built with React + Tailwind CSS.
          </>,
        ]}
      />

      <H2>Setup for development</H2>
      <H3>1. Clone the repository</H3>
      <Terminal
        lines={[
          "git clone https://github.com/pamod-madubashana/SoundCore-Desktop.git",
          "cd SoundCore-Desktop",
        ]}
      />
      <H3>2. Install root and UI dependencies</H3>
      <Terminal lines={["npm install", "npm --prefix ui install"]} />
      <H3>3. Install docs dependencies</H3>
      <Terminal lines={["cd docs", "npm install"]} />

      <H2>Working on the frontend</H2>
      <UL
        items={[
          <>
            Edit <Code>ui/src/App.jsx</Code> and <Code>ui/src/main.jsx</Code> for UI changes.
          </>,
          <>
            Update styles in <Code>ui/src/index.css</Code>.
          </>,
        ]}
      />

      <H2>Working on the backend</H2>
      <UL
        items={[
          <>
            Update Tauri integration in <Code>src-tauri/src/main.rs</Code>.
          </>,
          <>
            Modify device logic in <Code>src-tauri/src/worker.rs</Code>.
          </>,
          <>
            Adjust configuration parsing in <Code>src-tauri/src/config.rs</Code>.
          </>,
        ]}
      />

      <H2>Running the app during development</H2>
      <Terminal lines={["npm run tauri dev"]} />

      <H2>Building for release</H2>
      <Terminal lines={["npm run tauri build"]} />
      <P>This generates native installers for supported platforms.</P>

      <H2>Updating documentation</H2>
      <OL
        items={[
          <>
            Add or edit markdown files in <Code>docs/docs/</Code>.
          </>,
          <>
            Update <Code>docs/sidebars.js</Code> to include new content.
          </>,
          "Run the docs site locally with npm run start.",
        ]}
      />

      <H2>Contributing guidelines</H2>
      <UL
        items={[
          "Follow consistent naming and structure for new docs pages.",
          "Keep the UX simple and focused on restore behavior.",
          "Test changes on both Windows and Linux where possible.",
        ]}
      />
    </article>
  );
}
