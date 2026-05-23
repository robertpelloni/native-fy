# MEMORY

## Architectural Observations
- **Native Rendering Shell:** Leveraging `wgpu` for high-performance, cross-platform graphics. The pipeline uses instanced quad rendering for maximum efficiency in UI layouts.
- **Layout Engine:** Utilizing `taffy` (formerly `stretch`) for high-fidelity flexbox and grid layouts, matching modern web standards.
- **Scripting Layer:** `QuickJS` via `rquickjs` provides a lightweight, performant JavaScript environment without the overhead of a full browser engine.
- **Performance:** Native UI primitives consistently outperform JS-driven node creation, reducing bridge overhead for complex components.
- **Automation:** The "Autonomous Execution Protocol" ensures the codebase remains synchronized, tested, and self-healing across CI/CD cycles.
- **Memory Safety:** Implementation of LRU eviction policies for GPU resources (textures, text buffers) ensures stability during long-running sessions.
- **Lifecycle Management:** The engine now includes a background task scheduler in JS for health monitoring and maintenance.

## Design Preferences
- **Explicit Bridging:** Prefer explicit, batched UI commands over fine-grained, synchronous state mutations across the Rust/JS boundary.
- **Asset Pipeline:** Asynchronous, non-blocking asset loading is a core requirement to maintain 60 FPS UI performance.
- **Diagnostics:** The Native Monitoring Dashboard is the primary tool for real-time performance audit and debugging.
- **Stability:** Production mode (`PROD_MODE`) must always prioritize stability and silence over verbose logging.
