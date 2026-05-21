# CHANGELOG

## [0.5.0] - Unreleased

### Added
- Implemented Phase 4 native shell: Added a `winit` (v0.30) event loop and application handler in `src/main.rs` to create the physical desktop window.

## [0.4.0] - Unreleased

### Added
- Implemented Phase 3: The Compiler & Self-Healing Engine (`scripts/compiler_agent.js`).
- Created a self-healing LLM loop that reads a JSON AST, prompts the Gemini API to write the corresponding Rust `taffy` code into `src/main.rs`, runs `cargo check`, and feeds compiler errors back to the LLM until the code successfully compiles.
- Added `.env.example` placeholder file for securely storing `GEMINI_API_KEY`.
- Updated `DEPLOY.md` to reflect new required node environments and API key prerequisites.

## [0.3.0] - Unreleased

### Added
- Implemented Phase 2: Rust Core Runtime Utilities (`src/layout.rs`).
- Added strict `Node`, `AstRect`, and `FlexStyles` structs matching the Phase 1 AST schema.
- Added `LayoutEngine` struct integrating `taffy` to map the AST layout attributes directly to a Taffy flexbox tree.
- Implemented a strict validation pipeline inside `LayoutEngine::build_tree` that explicitly catches unsupported nodes or un-mappable CSS values and throws safe validation errors instead of crashing.

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