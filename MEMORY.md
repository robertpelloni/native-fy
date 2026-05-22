# MEMORY: Internal Observations & Design Preferences

## Architectural Observations
- **Wgpu Versioning:** `wgpu` v23 is the current stable anchor due to `glyphon` 0.7.0 compatibility.
- **QuickJS Bridge:** The `mpsc` queue for `UiCommand` is highly effective. It decouples the JS event loop from the rendering frame rate.
- **UI Generation:** Isolating generated code in `src/ui_gen.rs` prevents the AI from breaking core engine logic during self-healing loops.
- **Text Rendering:** `glyphon` is powerful but requires careful buffer management to avoid re-allocation spikes.

## Codebase Traits
- **Minimal Dependencies:** Preference for small, focused crates.
- **Safety:** Use of `std::panic::set_hook` to capture crashes in headless environments.
- **Logging:** Centralized timestamped logging in `app.log`.

## Design Preferences
- **Vanilla JS:** Avoid TS or complex build steps for internal scripts to keep the toolchain fast.
- **Explicit Bridging:** Prefer explicit command enums (`UiCommand`) over opaque FFI calls for UI mutations.
- **Performance First:** Instrumentation is built into the core `update` and `render` loops.
