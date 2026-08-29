import { DocTitle, H2, P, Code } from "../components/Prose";

const folders = [
  ["ui/", "React + Vite frontend for the tray UI and user interaction."],
  ["src-tauri/", "Rust backend with Tauri integration, system APIs, and device handling."],
  ["docs/", "Docusaurus documentation site and markdown content."],
  [".site/", "This website — built with React + Tailwind CSS."],
];

const rust = [
  ["src-tauri/src/main.rs", "Application launch and Tauri command registration."],
  ["src-tauri/src/worker.rs", "Device connection detection and profile restore logic."],
  ["src-tauri/src/config.rs", "Configuration parsing and app settings."],
  ["src-tauri/src/device_images.rs", "CDN image download, caching, and bundled image lookup."],
  ["src-tauri/src/autostart.rs", "Optional autostart implementation."],
];

const frontend = [
  ["ui/src/App.jsx", "Main React app entry point with all UI components."],
  ["ui/src/main.jsx", "React render logic."],
  ["ui/src/index.css", "Application styling."],
];

function FileTable({ rows }) {
  return (
    <div className="mt-4 max-w-[68ch] overflow-hidden rounded-lg border border-border">
      {rows.map(([path, body], i) => (
        <div
          key={path}
          className={`grid gap-1 px-4 py-3 sm:grid-cols-[minmax(0,15rem)_1fr] sm:items-baseline sm:gap-4 ${
            i > 0 ? "border-t border-border" : ""
          } bg-surface`}
        >
          <span className="font-mono text-[12.5px] text-primary">{path}</span>
          <span className="text-[13.5px] leading-relaxed text-muted-foreground">{body}</span>
        </div>
      ))}
    </div>
  );
}

export default function Architecture() {
  return (
    <article>
      <DocTitle
        eyebrow="docs / architecture"
        title="Architecture"
        lead="SoundCore-Desktop is structured as a desktop tray application with a React frontend and a Rust Tauri backend."
      />

      <H2>Core folders</H2>
      <FileTable rows={folders} />

      <H2>Main Rust components</H2>
      <FileTable rows={rust} />

      <H2>Frontend components</H2>
      <FileTable rows={frontend} />

      <H2>Deployment and packaging</H2>
      <P>
        The project is built using Tauri for native packaging, with <Code>npm run tauri build</Code>{" "}
        producing desktop installers.
      </P>
      <P>
        The docs site is a static Docusaurus site served from <Code>docs/build</Code> and published
        separately to GitHub Pages.
      </P>
    </article>
  );
}
