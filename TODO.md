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

## Scripting Runtime (QuickJS)
11. [DONE] Scaffold `QuickJS` integration in `src/runtime.rs` and `src/runtime.js`.
12. [DONE] Bind Rust UI events to JavaScript callbacks via a native bridge.
13. [ ] Implement a `fetch` polyfill for the QuickJS environment.

## Polish & Tooling
14. [DONE] Refactor `scripts/compiler_agent.js` to support separate UI module generation (`src/ui_gen.rs`).
15. [ ] Add automated UI integration tests using Playwright against the native window (via screenshot comparison).
