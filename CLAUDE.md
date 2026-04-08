# table-to-title — Claude instructions

Converts a biorxiv author submission TSV into a formatted title-page author line with affiliations, legend, and optional ORCiDs.

**Live site:** https://www.vollgerlab.com/table-to-title/
**Repo:** https://github.com/vollgerlab/table-to-title

## Two versions — always use the WASM one

| Version | Entry point | How it runs |
|---------|-------------|-------------|
| **WASM (preferred)** | `web/index.html` | Runs entirely in the browser; no server needed at runtime |
| Server | `static/index.html` + `src/main.rs` | Axum HTTP server; Excel upload handled server-side |

The WASM version is what gets deployed to GitHub Pages. Always build and preview the WASM version after making changes.

## After every code change

Rebuild WASM, copy artifacts, kill any stale server, start a fresh one, and open the browser:

```bash
wasm-pack build --target web --no-default-features --features wasm \
  && cp -r pkg/* web/pkg/ \
  && pkill -f "http.server 8080" 2>/dev/null; python3 -m http.server 8080 --directory web/ &
open http://localhost:8080
```

Step by step:

```bash
# 1. Build
wasm-pack build --target web --no-default-features --features wasm
cp -r pkg/* web/pkg/

# 2. Serve (WASM requires HTTP — file:// won't work due to ES module restrictions)
python3 -m http.server 8080 --directory web/

# 3. Open
open http://localhost:8080
```

## WASM build gotchas

- **Never pass `--out-dir` to wasm-pack** on stable Rust — it maps to cargo's `--artifact-dir` which is nightly-only. Build to the default `pkg/` then `cp -r pkg/* web/pkg/`.
- Always pass `--no-default-features --features wasm` — without this, tokio/mio are included via the `server` default feature and fail to compile to `wasm32-unknown-unknown`.
- `web/pkg/` is fully gitignored (`*`). Built artifacts are never committed; CI and local builds both create the directory fresh with `mkdir -p web/pkg`.

## Formatting conventions

- Superscript order: **affiliation numbers first, then symbols** — `1,2*†` not `*†1,2`.
- Equal-contribution marker: `*`
- Corresponding-author marker: `†` (U+2020)
- Legend lines come after affiliations, not before.

## GitHub Actions / GitHub Pages

Workflow: `.github/workflows/deploy.yml`

- Triggers on push to `main` and manual dispatch.
- Uses `Swatinem/rust-cache@v2` for Rust caching (hashes `Cargo.lock` automatically).
- Uses `jetli/wasm-pack-action@v0.4.0` to install wasm-pack from a pre-built binary (faster than curl|sh).
- Copies built artifacts with `mkdir -p web/pkg && cp -r pkg/* web/pkg/` — the `mkdir -p` is required because `web/pkg/` is gitignored and does not exist in a fresh checkout.
- Deploys `web/` via `actions/upload-pages-artifact` + `actions/deploy-pages` (OIDC-based, no `gh-pages` branch needed).
- GitHub Pages is configured to **Deploy from GitHub Actions** (Settings > Pages > Source).

## Running tests

```bash
cargo test
```
