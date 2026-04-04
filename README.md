# Spawn

One-click AI agent environment setup. Download, double-click, done.

## What It Does

A desktop wizard that gets a non-technical user from zero to a working AI agent environment:

1. **Free AI access** via Spawn Cloud proxy (no API key, no credit card)
2. **Hermes Agent** installed and configured automatically
3. **Paperclip** project management with pre-built company templates
4. **Project repo** cloned and ready

3 clicks. 0 terminals. 0 API keys.

## User Flow

```
Download -> Double-click -> Connect -> Pick Project -> Done
```

1. **Connect** -- auto-registers for free cloud AI access (50 messages/day)
2. **Pick a project** -- Geometry OS, ASCII World, AIPM
3. **Install** -- Hermes, Paperclip, project template, all configured
4. Run `hermes chat` to start

## Downloads

| Platform | File | Size |
|----------|------|------|
| Linux (Ubuntu/Debian) | `.deb` | ~3MB |
| Linux (universal) | `.AppImage` | ~74MB |
| Windows | `.msi` | TBD |
| macOS (Apple Silicon) | `.dmg` | TBD |
| macOS (Intel) | `.dmg` | TBD |

## Development

```bash
npm install
npm run dev            # Run in dev mode
npm run build:linux    # Build .deb + .AppImage
npm run build:windows  # Build .msi (needs Windows or CI)
npm run build:mac      # Build .dmg (needs macOS or CI)
```

## Cloud Dev Server

Test the cloud proxy locally:

```bash
cd cloud
node dev-server.js                              # Mock mode
OPENAI_API_KEY=sk-... node dev-server.js        # Real proxy mode
```

## Architecture

```
ui/             Setup wizard frontend (HTML/CSS/JS)
src-tauri/      Rust backend (Tauri)
  src/lib.rs    Commands: connect_cloud, run_setup, list_projects
cloud/          Cloud API (Cloudflare Worker)
  worker.js     Production: deploy with wrangler
  dev-server.js Local dev server
templates/      Bundled company export packages
  geometry-os/
  ascii-world/
```

## Deploy Cloud

```bash
cd cloud
wrangler login
wrangler kv:namespace create "KV"
# Put the namespace ID in wrangler.toml
wrangler secret put OPENAI_API_KEY
wrangler deploy
```

## Ship a Release

```bash
git tag v0.1.0
git push --tags
# GitHub Actions builds all platforms and creates a draft release
```

## License

MIT
