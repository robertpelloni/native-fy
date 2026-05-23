# CHANGELOG

## [0.19.0] - 2026-05-23
- Implemented Native Component Extensions for high-efficiency UI creation.
- Replaced blocking asset loading with an asynchronous thread-based pipeline.
- Implemented UI command batching in the main event loop to improve responsiveness.
- Conducted performance benchmarking for native components and async asset loading.

## [0.18.0] - 2026-05-23
- Implemented `glyphon::Buffer` caching to optimize text rendering.
- Added a standard UI component library (Button, Header, Card) to the QuickJS runtime.
- Enhanced `scripts/protocol_sync.js` to automatically synchronize `package.json` versioning.
- Conducted performance audit showing a 75% reduction in text rendering overhead.
