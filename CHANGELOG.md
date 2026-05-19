# CHANGELOG

## [0.2.0] - Unreleased

### Added
- Implemented Phase 1: Structural AST Extraction (`scripts/web_scraper.js`). The script now successfully navigates to URLs via Playwright, traverses the DOM, computes flexbox properties, and normalizes the UI layout into a structured JSON AST.

## [0.1.0] - Unreleased

### Added
- Initial project scaffolding: Node.js and Rust environments with `package.json` and `Cargo.toml`.
- Configured dependencies: `winit`, `wgpu`, `taffy` for Rust, and `playwright` for Node.js.
- Scaffolded basic source files (`web_scraper.js`, `layout.rs`, `main.rs`, `runtime.js`).
- Created initial `AGENTS.md` and related AI instruction files.
- Added comprehensive project documentation: `VISION.md`, `ROADMAP.md`, `TODO.md`, `DEPLOY.md`, `CHANGELOG.md`, `VERSION.md`, `HANDOFF.md`.