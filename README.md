# Paperclip Setup

One-click installer for Paperclip + Hermes AI agent setup.

## What It Does

A desktop wizard (Tauri) that sets up everything a new user needs:
1. **Free AI access** via Paperclip Cloud proxy (no API key needed)
2. **Hermes Agent** installed and configured automatically
3. **Paperclip** project management with pre-built company templates
4. **Project clone** ready to contribute

3 clicks. 0 terminals. 0 API keys.

## User Flow

1. Download the installer for your OS (.deb / .AppImage / .msi / .dmg)
2. Double-click to open
3. **Connect** -- auto-registers for free cloud AI access
4. **Pick a project** -- Geometry OS, ASCII World, or AIPM
5. **Install** -- Hermes, Paperclip, project template, all configured
6. Done. Run `hermes chat` to start.

## Development

```bash
# Install deps
npm install

# Run in dev mode (needs frontend dev server or static build)
npm run dev

# Build installers
npm run build:linux     # .deb + .AppImage
npm run build:windows   # .msi + .nsis (needs Windows or CI)
npm run build:mac       # .dmg + .app (needs macOS or CI)
```

## Cloud Dev Server

Test the cloud proxy locally:

```bash
cd cloud
node dev-server.js              # Mock mode (no OpenAI key)
OPENAI_API_KEY=sk-... node dev-server.js  # Real proxy mode
```

## Architecture

```
ui/           -- Setup wizard (HTML/CSS/JS, dark theme)
src-tauri/    -- Rust backend (process management, CLI orchestration)
  src/lib.rs  -- Commands: connect_cloud, run_setup, list_projects
cloud/        -- Cloudflare Worker (token issuer + OpenAI proxy)
  worker.js   -- Production: deploy with wrangler
  dev-server.js -- Local dev: node dev-server.js
templates/    -- Bundled company export packages
  geometry-os/  -- Geometry OS company template
  ascii-world/  -- ASCII World company template
```

## Deployment

1. Deploy cloud worker: `cd cloud && wrangler deploy`
2. Set secret: `wrangler secret put OPENAI_API_KEY`
3. Push a tag: `git tag v0.1.0 && git push --tags`
4. GitHub Actions builds installers for Linux, Windows, and macOS

## License

MIT
