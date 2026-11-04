# CHANGELOG

## [1.0.0-rc.1] - 2026-11-04
- **Sanitization & De-confliction:** Cleaned all merge conflict markers across Cargo.toml, Cargo.lock, package.json, VERSION.md, and source code.
- **Unified Control Dashboard:** Redesigned the monitoring dashboard into a consolidated single-page view featuring real-time ASCII telemetry gauges (FPS, CPU, RAM, GPU VRAM), latency breakdowns (Bridge, Layout, Render), and descriptive tooltips (`[?]`).
- **Server & Tray Lifecycle Management:** Added `UiCommand::ShutdownServers` and `NativeUI.shutdownServers()` to gracefully terminate the Python IPC socket listener and stop background tasks.
- **Python Bridge Expansion:** Added `shutdown()` to the Python client library and verified socket IPC control.
- **Script Archival & Cleanup:** Nondestructively archived redundant test runners into `scripts/archive/` while maintaining full E2E lifecycle test suites.
- **Headless & CI Reliability:** Fixed winit event loop panic handling and validated complete headless operation via `xvfb-run`.
