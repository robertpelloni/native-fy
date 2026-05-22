# CHANGELOG

## [0.13.0] - Unreleased

### Added
- Implemented robust system logging and error recovery infrastructure.
- Added custom panic hook to redirect application crashes to `app.log` with timestamps.
- Created `scripts/monitor.js` for automated stability and performance monitoring in live environments.
- Updated `DEPLOY.md` with release build and production monitoring instructions.
- Integrated `chrono` for advanced timestamping in logging.
- Refactored `RenderState` to use structured error propagation (`Result`) for better stability.

## [0.12.0] - Unreleased

### Added
- Implemented dynamic styling bridge between QuickJS and Rust.
- Added support for "Image" node type in `LayoutEngine` and the `wgpu` renderer.
- Integrated `image` crate for future texture decoding.
- Enhanced WGSL shader to support both colored primitives and textured nodes.
- Implemented node creation and child attachment in the main application loop via JS commands.

## [0.11.0] - Unreleased

### Added
- Implemented mpsc command queue for QuickJS-to-Native UI mutations.
- Added performance instrumentation for layout and rendering metrics.
- Benchmarked JS bridge (1000 nodes in 2.45ms).
- Created PERFORMANCE.md documenting initial baseline results.
- Added nativefy automation script to package.json.

## [0.10.0] - Unreleased

### Added
- Fully connected the QuickJS runtime to the application lifecycle.
- Implemented basic input bridging: Mouse cursor tracking and click event dispatching to JavaScript.
- Added `JsRuntime::dispatch_click` for safe event propagation from Rust to JS.
- Refined `runtime.js` to support an `addEventListener` pattern for application logic.

## [0.9.0] - Unreleased

### Added
- Integrated `QuickJS` (via `rquickjs`) as the application scripting engine.
- Implemented a native-to-JavaScript bridge in `src/runtime.rs` and `src/runtime.js`.
- Exposed `NativeUI` global object to JS for node creation and styling.
- Added `eval` method to `JsRuntime` for executing JavaScript application logic.

## [0.8.0] - Unreleased

### Added
- Integrated `glyphon` for high-performance native text rendering.
- Updated `LayoutEngine` to store and expose text content from the AST.
- Implemented a text render pass that handles preparation and rendering of text areas.
- Aligned `wgpu` and `glyphon` versions to ensure API compatibility (using wgpu v23).
- Added `cosmic-text` support via glyphon for advanced text shaping.

## [0.7.0] - Unreleased

### Added
- Implemented a quad renderer using `wgpu` and instanced drawing.
- Created a WGSL shader for efficient rendering of UI layout nodes.
- Integrated `taffy` layout results directly into the `wgpu` render pass.
- Added `bytemuck` dependency for safe memory mapping between Rust and the GPU.
- Implemented a coordinate transformation system to map Taffy screen-space pixels to wgpu Normalized Device Coordinates (NDC).

## [0.6.0] - Unreleased

### Added
- Initialized `wgpu` rendering context in `src/main.rs`.
- Implemented foundational `wgpu` lifecycle: Instance, Adapter, Device, Queue, and Surface creation.
- Added window resizing and basic redraw handling (clearing the screen).
- Expanded `ROADMAP.md` and `TODO.md` to reflect Phase 5 (Rendering) and Phase 6 (QuickJS).
- Created `MEMORY.md` to track long-term architectural decisions.
- Created `IDEAS.md` for project expansion brainstorming.

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
