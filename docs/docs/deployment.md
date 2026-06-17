---
id: deployment
title: Deployment
---

# Deployment

## GitHub Pages

This documentation site can be deployed directly to GitHub Pages from the `gh-pages` branch.

### Build and deploy locally

```bash
cd docs
npm install
npm run build
npx docusaurus deploy
```

### GitHub Actions

A workflow can build the site and publish `docs/build` to GitHub Pages automatically whenever the `main` branch changes.

## Alternative hosts

- Cloudflare Pages
- Vercel
- GitHub Pages

For static site deployment, build the site with `npm run build` and upload the generated `build` folder.
