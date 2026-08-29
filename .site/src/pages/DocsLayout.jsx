import { Link, Outlet, useLocation } from "react-router-dom";
import { SiteHeader } from "../components/SiteHeader";
import { SiteFooter } from "../components/SiteFooter";

const sections = [
  {
    title: "Getting started",
    links: [
      { to: "/docs", label: "Introduction", exact: true },
      { to: "/docs/installation", label: "Installation" },
      { to: "/docs/features", label: "Features" },
      { to: "/docs/usage", label: "Usage" },
    ],
  },
  {
    title: "Reference",
    links: [
      { to: "/docs/architecture", label: "Architecture" },
      { to: "/docs/development", label: "Development" },
      { to: "/docs/deployment", label: "Deployment" },
      { to: "/docs/troubleshooting", label: "Troubleshooting" },
    ],
  },
];

export default function DocsLayout() {
  const location = useLocation();

  return (
    <div className="min-h-screen">
      <SiteHeader />
      <div className="mx-auto grid max-w-6xl gap-12 px-6 py-14 md:grid-cols-[220px_1fr]">
        <aside className="md:sticky md:top-24 md:self-start">
          <div className="space-y-7">
            {sections.map((section) => (
              <div key={section.title}>
                <p className="font-mono text-[10.5px] uppercase tracking-[0.2em] text-faint">
                  {section.title}
                </p>
                <nav className="mt-3 space-y-0.5 border-l border-border font-mono text-[13px]">
                  {section.links.map((link) => {
                    const isActive = link.exact
                      ? location.pathname === link.to
                      : location.pathname.startsWith(link.to);
                    return (
                      <Link
                        key={link.to}
                        to={link.to}
                        className={`block border-l border-transparent py-1 pl-4 transition-colors hover:text-foreground ${
                          isActive
                            ? "border-primary text-primary"
                            : "text-muted-foreground"
                        }`}
                      >
                        {link.label}
                      </Link>
                    );
                  })}
                </nav>
              </div>
            ))}
            <div>
              <p className="font-mono text-[10.5px] uppercase tracking-[0.2em] text-faint">Get it</p>
              <nav className="mt-3 space-y-0.5 border-l border-border font-mono text-[13px]">
                <Link
                  to="/downloads"
                  className="block border-l border-transparent py-1 pl-4 text-muted-foreground transition-colors hover:text-foreground"
                >
                  Downloads
                </Link>
              </nav>
            </div>
          </div>
        </aside>

        <main className="min-w-0">
          <Outlet />
        </main>
      </div>
      <SiteFooter />
    </div>
  );
}
