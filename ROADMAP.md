# ROADMAP

## Phase 1: Structural AST Extraction [DONE]
Implement a Playwright script (`scripts/web_scraper.js`) that navigates to URLs and extracts layout topology, ignoring web clutter. It outputs a normalized JSON Abstract Syntax Tree (AST) focusing on structural boxes, coordinates, padding, margins, flexbox properties, text values, and inputs.

## Phase 2: Rust Core Runtime Utilities [DONE]
Implement data structures in Rust (`src/layout.rs`) that directly map the extracted structural JSON AST to Taffy flexbox layout nodes, enforcing strict type safety to reject any properties outside the supported subset.

## Phase 3: Compiler & Self-Healing Engine [DONE]
Create an automated agent script to handle the LLM generation loop, translating the JSON AST into the native `wgpu` + `taffy` Rust UI tree. This will involve an isolated execution loop running `cargo check/test` and feeding compiler errors back to the model for self-correction.

## Phase 4: Native Shell & Windowing [DONE]
Integrate `winit` (v0.30) to provide a cross-platform windowing shell. Handle the `ApplicationHandler` lifecycle to manage window creation and event processing.

## Phase 5: Hardware-Accelerated Rendering [IN PROGRESS]
Integrate `wgpu` to provide a high-performance, GPU-accelerated rendering backend.
- Initialize `wgpu` Instance, Adapter, Device, Queue, and Surface.
- Implement a basic render pass to clear the screen and handle window resizing.
- Develop a primitive scene graph to draw layout nodes (rectangles, text, etc.) calculated by Taffy.

## Phase 6: QuickJS Runtime Bridge
Integrate `QuickJS` to allow JavaScript-based business logic to interact with the native Rust UI.
- Expose Rust-native UI controls to the JS environment.
- Implement an event-driven communication layer between JS and Rust.
