# table-to-title

Converts a biorxiv author submission TSV into a formatted title-page author line with affiliations, legend, and optional ORCiDs.

## Two versions — always use the WASM one

| Version | Entry point | How it runs |
|---------|-------------|-------------|
| **WASM (preferred)** | `web/index.html` | Runs entirely in the browser; no server needed at runtime |
| Server | `static/index.html` + `src/main.rs` | Axum HTTP server; Excel upload handled server-side |

The WASM version is what gets deployed to GitHub Pages and is what the user always wants to see.

## After every code change

Rebuild the WASM and open a local preview:

```bash
wasm-pack build --target web --no-default-features --features wasm \
  && cp -r pkg/* web/pkg/ \
  && pkill -f "http.server 8080" 2>/dev/null; python3 -m http.server 8080 --directory web/ &
open http://localhost:8080
```

Or step by step:

```bash
# 1. Build
wasm-pack build --target web --no-default-features --features wasm
cp -r pkg/* web/pkg/

# 2. Serve (WASM requires HTTP — file:// won't work)
python3 -m http.server 8080 --directory web/

# 3. Open
open http://localhost:8080
```

## WASM build gotchas

- **Never pass `--out-dir` to wasm-pack** on stable Rust — it maps to cargo's `--artifact-dir` which is nightly-only. Build to the default `pkg/` and then `cp -r pkg/* web/pkg/`.
- Disable the `server` feature when building for WASM: `--no-default-features --features wasm`. Without this, tokio/mio pull in and fail to compile to `wasm32-unknown-unknown`.
- `web/pkg/` is fully gitignored (`*`). The built artifacts are never committed; CI rebuilds them fresh.

## Formatting conventions

- Superscript order: **affiliation numbers first, then symbols** — e.g. `1,2*†` not `*†1,2`.
- Equal-contribution marker: `*`
- Corresponding-author marker: `†` (U+2020)
- Legend comes after affiliations, not before.

## GitHub Actions / GitHub Pages

Workflow: `.github/workflows/deploy.yml`

- Triggers on push to `main` and manual dispatch.
- Uses `Swatinem/rust-cache@v2` for Rust caching (hashes `Cargo.lock` automatically).
- Uses `jetli/wasm-pack-action` to install wasm-pack from a pre-built binary (fast).
- Deploys `web/` via `actions/upload-pages-artifact` + `actions/deploy-pages` (OIDC-based, no `gh-pages` branch).
- In repo Settings > Pages > Source, set to **GitHub Actions**.

## Running tests

```bash
cargo test
```
