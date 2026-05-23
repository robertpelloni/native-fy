# CHANGELOG

## [0.24.0] - 2026-05-23
- Implemented an Autonomous Task Scheduler in the JS runtime.
- Exposed the automation pipeline to the engine via `NativeUI.runPipeline()`.
- Enhanced `scripts/e2e_test.js` to include autonomous self-check and maintenance validation.
- Stabilized the runtime for long-term autonomous maintenance.

## [0.23.0] - 2026-05-23
- Implemented robust texture management and dynamic image loading.
- Enhanced the rendering pipeline with a texture batching system to support unique textures per node.
- Added `NativeUI.reload()` to the JS runtime for live UI tree re-generation.
- Improved `UpdateImage` command handler with RGBA decoding and GPU texture uploads.
