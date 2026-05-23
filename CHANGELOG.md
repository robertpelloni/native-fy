# CHANGELOG

## [0.23.0] - 2026-05-23
- Implemented robust texture management and dynamic image loading.
- Enhanced the rendering pipeline with a texture batching system to support unique textures per node.
- Added `NativeUI.reload()` to the JS runtime for live UI tree re-generation.
- Improved `UpdateImage` command handler with RGBA decoding and GPU texture uploads.
- Optimized multi-texture rendering using batched draw calls and bind group switching.

## [0.22.0] - 2026-05-23
- Implemented Full Automated E2E Lifecycle validation (`scripts/e2e_test.js`).
- Integrated `PROD_MODE` to silence debug overlays for clean production deployment.
- Enhanced `BENCHMARK_MODE` for immediate metric export in headless environments.
- Formally established the "start" and "test:e2e" production scripts in `package.json`.
