import { Link } from "react-router-dom";

const GITHUB = "https://github.com/pamod-madubashana/SoundCore-Desktop";

export function SiteFooter() {
  return (
    <footer className="border-t border-border">
      <div className="mx-auto flex max-w-6xl flex-col gap-5 px-6 py-10 md:flex-row md:items-center md:justify-between">
        <div className="flex items-center gap-2.5">
          <span className="grid size-6 place-items-center rounded-[4px] border border-primary/40 bg-primary/10 font-mono text-[11px] leading-none text-primary">
            SC
          </span>
          <span className="text-sm font-semibold tracking-tight">SoundCore-Desktop</span>
          <span className="font-mono text-[11px] text-faint">open source · no mobile app needed</span>
        </div>
        <nav className="flex flex-wrap gap-x-6 gap-y-2 font-mono text-[12px] text-muted-foreground">
          <Link to="/docs" className="transition-colors hover:text-foreground">
            Docs
          </Link>
          <Link to="/docs/installation" className="transition-colors hover:text-foreground">
            Installation
          </Link>
          <Link to="/downloads" className="transition-colors hover:text-foreground">
            Downloads
          </Link>
          <a href={GITHUB} className="transition-colors hover:text-foreground">
            GitHub
          </a>
          <a href={`${GITHUB}/issues`} className="transition-colors hover:text-foreground">
            Issues
          </a>
        </nav>
      </div>
    </footer>
  );
}
