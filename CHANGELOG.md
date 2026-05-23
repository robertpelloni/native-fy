# CHANGELOG

## [0.16.0] - 2026-05-23
- Integrated the Autonomous Execution Protocol into the Rust build workflow (`build.rs`).
- Implemented dynamic storage buffer resizing for UI nodes to support arbitrary layout complexity.
- Stabilized the end-to-end autonomous pipeline (Sync -> Build -> Monitor).
- Synchronized all metadata and documentation to version 0.16.0.

## [0.15.0] - 2026-05-22
- Implemented live Performance & Protocol Status Overlay in the primary application.
- Integrated `reqwest` and implemented `fetch` polyfill in the QuickJS runtime.
- Updated `compiler_agent.js` system prompt to align with UI protocol standards.
- Finalized Phase 3: Asset management & Networking milestones.

## [0.14.0] - 2026-05-22
- Integrated the Autonomous Execution Protocol into the core pipeline.
- Created `scripts/protocol_sync.js` for automated documentation and versioning management.
- Updated `package.json` with a unified `pipeline` script for end-to-end automation.
- Conducted end-to-end performance validation and updated `PERFORMANCE.md`.

## [0.13.0] - 2026-05-22
- Implemented robust panic handling and timestamped logging in `main.rs`.
- Created `scripts/monitor.js` for real-time performance and stability tracking.
- Added comprehensive instrumentation for layout and rendering frame-times.
