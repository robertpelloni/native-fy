# Native-fy

Native-fy is an experimental Rust desktop UI runtime that skips web engines
entirely: layout trees are compiled to a native scene graph rendered with
`wgpu`, laid out with `taffy` (flexbox), and scripted through QuickJS
(`rquickjs`) and a Python bridge (`src/python_bridge.rs`). No Chromium, no
WebKit, no DOM — UI is built from native node primitives driven over an
MPSC `UiCommand` channel.

## What's in the tree

- `src/` — real Rust code: `app.rs` (~43 KB), `render.rs` (~40 KB,
  instanced wgpu pipeline, `shader.wgsl`), `runtime.rs` + `runtime.js`
  (QuickJS host), `layout.rs`, `monitor.rs` (telemetry dashboard),
  `hot_reload.rs`, `python_bridge.rs`, `ui_gen.rs`
- `Cargo.toml` — deps as above plus `winit`, `glyphon`, `image`, `reqwest`
- `scripts/` + `package.json` — a Node/Playwright "transpiler" pipeline:
  scrape a page's layout AST, feed it to an LLM (Gemini), generate Rust UI
  code, and loop on `cargo check` errors until it builds
- `python/`, `prompts/`, `staging/` — pipeline support material

## ⚠️ The repo is committed mid-merge-conflict

This is not a figure of speech. `README.md` (the file this replaces),
`VERSION.md`, **and `Cargo.toml`** all contain literal
`<<<<<<< HEAD` / `=======` / `>>>>>>>` conflict markers at HEAD, between a
"1.0.0-rc.1" line and a "0.40.0 / 0.38.0" line from a Jules branch. So:

- The project's version is indeterminate from the repo as committed
  (the two sides claim 1.0.0-rc.1 and 0.38–0.40).
- `cargo build` will fail until `Cargo.toml` is de-conflicted.
- Claims from the conflicted README sides ("v1.0.0 Release Candidate,
  autonomous pipeline fully functional", "single binary under 10 MB at
  60+ FPS") are unverified aspirations from the two branches, not facts
  about HEAD.

## Running (once de-conflicted)

```bash
cargo run --release
npm install && npx playwright install   # for the transpiler pipeline
```

See `DEPLOY.md` for the pipeline details and `AGENTS.md` for the strict
architecture rules (no DOM elements, `AstRect` primitives only).

## Status

Active-prototype state, developed largely by autonomous agents (Jules
branches, checkpoint commits through October 2026). Interesting and
substantial code; currently unbuildable as committed due to the merge
state described above.
