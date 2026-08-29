export function DocTitle({ eyebrow, title, lead }) {
  return (
    <header className="mb-10 border-b border-border pb-8">
      <p className="eyebrow">{eyebrow}</p>
      <h1 className="mt-3 text-3xl font-semibold tracking-tight text-balance sm:text-[2.25rem]">
        {title}
      </h1>
      {lead && (
        <p className="mt-4 max-w-[62ch] text-[15px] leading-relaxed text-muted-foreground text-pretty">
          {lead}
        </p>
      )}
    </header>
  );
}

export function H2({ children }) {
  return (
    <h2 className="mt-12 mb-3 text-[1.15rem] font-semibold tracking-tight first:mt-0">{children}</h2>
  );
}

export function H3({ children }) {
  return <h3 className="mt-8 mb-2 text-[15px] font-semibold tracking-tight">{children}</h3>;
}

export function P({ children }) {
  return (
    <p className="mt-3 max-w-[68ch] text-[14.5px] leading-[1.75] text-muted-foreground">{children}</p>
  );
}

export function UL({ items }) {
  return (
    <ul className="mt-4 max-w-[68ch] space-y-2">
      {items.map((item, i) => (
        <li key={i} className="flex gap-3 text-[14.5px] leading-relaxed text-muted-foreground">
          <span className="mt-2 size-1 shrink-0 rounded-full bg-primary" />
          <span>{item}</span>
        </li>
      ))}
    </ul>
  );
}

export function OL({ items }) {
  return (
    <ol className="mt-4 max-w-[68ch] space-y-3">
      {items.map((item, i) => (
        <li key={i} className="flex gap-3 text-[14.5px] leading-relaxed text-muted-foreground">
          <span className="mt-0.5 font-mono text-[12px] text-primary">
            {String(i + 1).padStart(2, "0")}
          </span>
          <span>{item}</span>
        </li>
      ))}
    </ol>
  );
}

export function Code({ children }) {
  return (
    <code className="rounded-[4px] border border-border bg-surface px-1.5 py-0.5 font-mono text-[12.5px] text-foreground">
      {children}
    </code>
  );
}

export function Terminal({ title = "bash", lines }) {
  return (
    <div className="mt-4 max-w-[68ch] overflow-hidden rounded-lg border border-border bg-surface">
      <div className="flex items-center gap-2 border-b border-border px-4 py-2.5">
        <span className="size-2.5 rounded-full bg-faint/50" />
        <span className="size-2.5 rounded-full bg-faint/50" />
        <span className="size-2.5 rounded-full bg-primary/70" />
        <span className="ml-2 font-mono text-[11px] text-faint">{title}</span>
      </div>
      <pre className="overflow-x-auto px-4 py-4 font-mono text-[12.5px] leading-[1.8]">
        <code>
          {lines.map((line, i) => (
            <span key={i} className="block">
              {line.startsWith("#") ? (
                <span className="text-faint">{line}</span>
              ) : (
                <>
                  <span className="text-faint">$ </span>
                  <span className="text-foreground">{line}</span>
                </>
              )}
            </span>
          ))}
        </code>
      </pre>
    </div>
  );
}

export function Note({ children }) {
  return (
    <div className="mt-6 max-w-[68ch] rounded-lg border border-primary/25 bg-primary/[0.06] px-4 py-3 text-[13.5px] leading-relaxed text-muted-foreground">
      {children}
    </div>
  );
}
