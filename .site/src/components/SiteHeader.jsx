import { Link, useLocation } from "react-router-dom";
import appIcon from "../assets/icon.svg";

const GITHUB = "https://github.com/pamod-madubashana/SoundCore-Desktop";

const nav = [
  { label: "Docs", to: "/docs" },
  { label: "Features", to: "/docs/features" },
  { label: "Usage", to: "/docs/usage" },
  { label: "Downloads", to: "/downloads" },
];

export function SiteHeader() {
  const location = useLocation();

  return (
    <header className="sticky top-0 z-50 border-b border-border bg-background/85 backdrop-blur">
      <div className="mx-auto flex max-w-6xl items-center justify-between px-6 py-3.5">
        <Link to="/" className="flex items-center gap-2.5">
          <img src={appIcon} alt="" className="size-6" />
          <span className="text-sm font-semibold tracking-tight">SoundCore-Desktop</span>
        </Link>

        <nav className="hidden items-center gap-7 font-mono text-[12.5px] text-muted-foreground md:flex">
          {nav.map((item) => (
            <Link
              key={item.to}
              to={item.to}
              className={`transition-colors hover:text-foreground ${
                location.pathname === item.to || location.pathname === item.to.replace(/\/$/, "")
                  ? "text-foreground"
                  : ""
              }`}
            >
              {item.label}
            </Link>
          ))}
        </nav>

        <div className="flex items-center gap-2.5">
          <a
            href={GITHUB}
            className="hidden rounded-md border border-border px-3 py-1.5 font-mono text-[12px] text-muted-foreground transition-colors hover:border-foreground/30 hover:text-foreground sm:block"
          >
            GitHub
          </a>
          <Link
            to="/downloads"
            className="rounded-md border border-primary/50 bg-primary/10 px-3 py-1.5 font-mono text-[12px] font-medium text-primary transition-colors hover:bg-primary/20"
          >
            Download
          </Link>
        </div>
      </div>
    </header>
  );
}
