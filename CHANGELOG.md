# CHANGELOG

## [0.18.0] - 2026-05-23
- Implemented `glyphon::Buffer` caching to optimize text rendering.
- Added a standard UI component library (Button, Header, Card) to the QuickJS runtime.
- Enhanced `scripts/protocol_sync.js` to automatically synchronize `package.json` versioning.
- Conducted performance audit showing a 75% reduction in text rendering overhead.

## [0.17.0] - 2026-05-23
- Integrated administrative protocol commands (`syncProtocol`, `getMetadata`) into the JS runtime.
- Updated the application event loop to handle documentation synchronization asynchronously.
- Enhanced the debug overlay with an "AUTO-SYNC" status indicator.
- Implemented embedded metadata retrieval using Rust's `include_str!` macro.

## [0.16.0] - 2026-05-23
- Integrated the Autonomous Execution Protocol into the Rust build workflow (`build.rs`).
- Implemented dynamic storage buffer resizing for UI nodes to support arbitrary layout complexity.
- Stabilized the end-to-end autonomous pipeline (Sync -> Build -> Monitor).
