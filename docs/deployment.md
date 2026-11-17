---
id: deployment
title: Deployment
---

# Deployment

## GitHub Pages

The marketing site lives under `.site/` (own `package.json` for the Pages workflow only — not part of the Rust app).

### GitHub Actions

`.github/workflows/deploy-site.yml` runs `npm ci` + `npm run build` inside `.site/` on pushes to `main` that touch `.site/**`.

## Alternative hosts

- Cloudflare Pages
- Vercel
- GitHub Pages

Build the site from `.site/` and upload the generated `dist` folder.
