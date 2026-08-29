import { DocTitle, H2, H3, P, UL, Terminal, Code } from "../components/Prose";

export default function Deployment() {
  return (
    <article>
      <DocTitle
        eyebrow="docs / deployment"
        title="Deployment"
        lead="How the documentation site is built and published."
      />

      <H2>GitHub Pages</H2>
      <P>
        This documentation site can be deployed directly to GitHub Pages from the{" "}
        <Code>gh-pages</Code> branch.
      </P>

      <H3>Build and deploy locally</H3>
      <Terminal lines={["cd docs", "npm install", "npm run build", "npx docusaurus deploy"]} />

      <H3>GitHub Actions</H3>
      <P>
        A workflow can build the site and publish <Code>docs/build</Code> to GitHub Pages
        automatically whenever the <Code>main</Code> branch changes.
      </P>

      <H2>Alternative hosts</H2>
      <UL items={["Cloudflare Pages", "Vercel", "GitHub Pages"]} />
      <P>
        For static site deployment, build the site with <Code>npm run build</Code> and upload the
        generated <Code>build</Code> folder.
      </P>
    </article>
  );
}
