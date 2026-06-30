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

## Stability & Resource Guarantees (Phase 5)
- **Watchdog Recovery Protocols:** The system executes an explicit Node.js validation pass on initial boot (`runE2eLifecycleValidation`). If `fps` consistently drops below 5, it considers the execution loop stalled and autonomously triggers the watchdog recovery sequence to restore fidelity.
- **Cache Eviction Thresholds:** The native auto-scaler limits standard text and texture allocation pools dynamically based on OS-level `sysinfo`. Default text thresholds scale between `50` and `1000` buffered glyphs depending on memory bounds.
