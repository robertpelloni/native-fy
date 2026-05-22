# ROADMAP

## Phase 1: Structural AST Extraction [DONE]
Implement a Playwright script (`scripts/web_scraper.js`) that navigates to URLs and extracts layout topology, ignoring web clutter. It outputs a normalized JSON Abstract Syntax Tree (AST) focusing on structural boxes, coordinates, padding, margins, flexbox properties, text values, and inputs.

## Phase 2: Rust Core Runtime Utilities [DONE]
Implement data structures in Rust (`src/layout.rs`) that directly map the extracted structural JSON AST to Taffy flexbox layout nodes, enforcing strict type safety to reject any properties outside the supported subset.

## Phase 3: Compiler & Self-Healing Engine [DONE]
Create an automated agent script to handle the LLM generation loop, translating the JSON AST into the native `wgpu` + `taffy` Rust UI tree. This will involve an isolated execution loop running `cargo check/test` and feeding compiler errors back to the model for self-correction.

## Phase 4: Native Shell & Windowing [DONE]
Integrate `winit` (v0.30) to provide a cross-platform windowing shell. Handle the `ApplicationHandler` lifecycle to manage window creation and event processing.

## Phase 5: Hardware-Accelerated Rendering [DONE]
Integrate `wgpu` to provide a high-performance, GPU-accelerated rendering backend.
- [DONE] Initialize `wgpu` Instance, Adapter, Device, Queue, and Surface.
- [DONE] Implement a basic render pass to clear the screen and handle window resizing.
- [DONE] Develop a primitive scene graph to draw layout nodes (rectangles, text, etc.) calculated by Taffy.

## Phase 6: QuickJS Runtime Bridge [DONE]
Integrate `QuickJS` to allow JavaScript-based business logic to interact with the native Rust UI.
- [DONE] Expose Rust-native UI controls to the JS environment.
- [DONE] Implement an event-driven communication layer between JS and Rust.

## Phase 7: Application Lifecycle & Interactivity [DONE]
Connect all engine components to provide a cohesive application environment.
- [DONE] Connect `winit` input events to QuickJS.
- [DONE] Conduct initial performance validation and bridge benchmarking.
- [DONE] Implement "Image" node support in the layout and rendering pipeline.
- [DONE] Establish robust error logging and system monitoring.
- [ ] Implement a `fetch` polyfill for QuickJS.

## Phase 8: Optimization & Scaling
- [ ] Implement dynamically resizing GPU buffers.
- [ ] Add sophisticated component caching in the text renderer.
- [ ] Explore SDF-based rendering for advanced UI elements (rounded corners, shadows).
