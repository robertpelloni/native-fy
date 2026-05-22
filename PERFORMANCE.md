# PERFORMANCE: Native-fy Benchmark Results

## JS-to-Native Bridge (QuickJS)
- **Node Creation:** 1000 nodes in ~2.45ms (Average)
- **Command Overhead:** ~2.4µs per `UiCommand`

## Layout Engine (Taffy)
- **Initial Compute:** ~450µs for 100 nodes
- **Re-compute (Partial):** ~120µs

## Rendering Pipeline (wgpu)
- **Frame Time:** ~1.2ms (1000 quads + basic text)
- **GPU Upload:** ~300µs for 1024 node storage buffer

## End-to-End Pipeline (v0.14.0)
- **Protocol Sync:** ~50ms
- **AST Extraction (Playwright):** ~2-5s (Site dependent)
- **AI Compilation (Gemini):** ~5-15s
- **Rust Compilation:** ~0.2s (incremental)
- **Runtime Startup:** ~80ms (Headless failure overhead included)
