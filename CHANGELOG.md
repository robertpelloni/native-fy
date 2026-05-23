# CHANGELOG

## [0.25.0] - 2026-05-23
- Implemented Cache Eviction Policies for text buffers and textures to prevent memory leaks.
- Integrated Frame Capture (Screenshot) API for automated visual testing.
- Created an Automated Visual Regression Suite (`scripts/visual_test.js`).
- Integrated visual regression validation into the unified autonomous pipeline.

## [0.24.0] - 2026-05-23
- Implemented an Autonomous Task Scheduler in the JS runtime.
- Exposed the automation pipeline to the engine via `NativeUI.runPipeline()`.
- Enhanced `scripts/e2e_test.js` to include autonomous self-check and maintenance validation.
