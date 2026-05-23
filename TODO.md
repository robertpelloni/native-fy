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
12. [DONE] Implement live Performance & Protocol Status Overlay.
13. [DONE] Implement dynamic storage buffer resizing for nodes.
14. [DONE] Optimize text rendering with glyphon buffer caching.
15. [DONE] Implement Native Monitoring Dashboard (`DASHBOARD_MODE`).
16. [DONE] Implement production mode silence logic (`PROD_MODE`).
17. [DONE] Implement Dynamic Texture Management and GPU Uploads.

## Scripting Runtime (QuickJS)
18. [DONE] Scaffold `QuickJS` integration in `src/runtime.rs` and `src/runtime.js`.
19. [DONE] Bind Rust UI events to JavaScript callbacks via a native bridge.
20. [DONE] Implement dynamic styling bridge for JS-to-Native property mapping.
21. [DONE] Implement a `fetch` polyfill for the QuickJS environment.
22. [DONE] Integrate Autonomous Protocol Sync into the JS runtime.
23. [DONE] Implement standard component library abstractions (Button, Card, etc.).
24. [DONE] Implement Native Component Extensions for performance optimization.
25. [DONE] Implement bridge health checks and heartbeats.
26. [DONE] Implement `NativeUI.reload()` for live re-generation.
27. [DONE] Implement Autonomous Task Scheduler (JS).
28. [DONE] Implement `NativeUI.runPipeline()` for engine-triggered automation.

## Performance & Stability
29. [DONE] Instrument core engine for frame-time and layout-time measurement.
30. [DONE] Conduct benchmark of JS-to-Native bridge (1000 nodes in < 3ms).
31. [DONE] Implement robust panic handling and disk-based error logging.
32. [DONE] Create automated stability monitoring script for live deployment.
33. [DONE] Integrate Autonomous Execution Protocol into the core pipeline.
34. [DONE] Integrate Protocol Sync into the Rust build workflow (`build.rs`).
35. [DONE] Implement Asynchronous Asset Loading pipeline.
36. [DONE] Implement UI Command Batching.
37. [DONE] Implement automated performance benchmarking and report generation.
38. [DONE] Implement Continuous Health Watchdog (`health_monitor.js`).
39. [DONE] Implement Full End-to-End lifecycle validation (`e2e_test.js`).

## Polish & Tooling
40. [DONE] Refactor `scripts/compiler_agent.js` to support separate UI module generation (`src/ui_gen.rs`).
41. [DONE] Add `nativefy` convenience script to `package.json`.
42. [ ] Add automated UI integration tests using Playwright against the native window (via screenshot comparison).
