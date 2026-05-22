# TODO

## Core Foundation
1. [DONE] Initialize project scaffolding (Rust and Node setup)
2. [DONE] Implement Phase 1: Playwright web scraper for Structural AST Extraction
3. [DONE] Implement Phase 2: Rust data structures and Taffy mapping
4. [DONE] Implement Phase 3: Compiler and self-healing LLM loop
5. [DONE] Wire the winit layer to create a native desktop window.

## Rendering Engine (wgpu)
6. [DONE] Initialize `wgpu` context (Instance, Adapter, Device, Queue, Surface) in `src/main.rs`.
7. [DONE] Implement `WindowEvent::Resized` to handle surface reconfiguration.
8. [DONE] Implement basic `WindowEvent::RedrawRequested` to clear the screen with a solid color.
9. [DONE] Create a "Box" renderer to draw rectangles based on Taffy layout coordinates using instanced drawing.
10. [DONE] Implement a "Text" renderer using `glyphon` for high-performance font rendering.
11. [DONE] Implement "Image" rendering support in the `wgpu` pipeline.

## Scripting Runtime (QuickJS)
12. [DONE] Scaffold `QuickJS` integration in `src/runtime.rs` and `src/runtime.js`.
13. [DONE] Bind Rust UI events to JavaScript callbacks via a native bridge.
14. [DONE] Implement dynamic styling bridge for JS-to-Native property mapping.
15. [ ] Implement a `fetch` polyfill for the QuickJS environment.

## Performance & Stability
16. [DONE] Instrument core engine for frame-time and layout-time measurement.
17. [DONE] Conduct benchmark of JS-to-Native bridge (1000 nodes in < 3ms).
18. [DONE] Implement robust panic handling and disk-based error logging.
19. [DONE] Create automated stability monitoring script for live deployment.
20. [ ] Implement dynamic storage buffer resizing for nodes.

## Polish & Tooling
21. [DONE] Refactor `scripts/compiler_agent.js` to support separate UI module generation (`src/ui_gen.rs`).
22. [DONE] Add `nativefy` convenience script to `package.json`.
23. [ ] Add automated UI integration tests using Playwright against the native window (via screenshot comparison).
