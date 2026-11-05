# Native-fy UI Engine

**Version:** 1.0.0-rc.1
Native-fy is an experimental Rust desktop UI runtime that skips web engines entirely: layout trees are compiled to a native scene graph rendered with `wgpu`, laid out with `taffy` (flexbox), and scripted through QuickJS (`rquickjs`) with a high-performance Python IPC bridge (`src/python_bridge.rs`). No Chromium, no WebKit, no DOM — UI is built from native node primitives driven over an MPSC `UiCommand` channel.

## Architecture Highlights
- **wgpu Scene Graph:** Low-level, hardware-accelerated instanced quad rendering pipeline with glyphon font rendering and tiny-skia SVG vector rasterization.
- **Taffy Flexbox Engine:** Compliant Flexbox layout computation directly in Rust.
- **QuickJS Runtime:** Ultra-lightweight JavaScript host communicating via non-blocking channels with the main loop.
- **Consolidated Control Dashboard:** Single-page live telemetry dashboard with real-time ASCII performance meters (CPU, FPS, RAM, GPU VRAM), latency diagnostics (Bridge, Layout, Render), and service status indicators with tooltips (`[?]`).
- **Python IPC Bridge & Server Lifecycle:** Standalone socket server on port 9090 allowing external automation, snapshot capture, dashboard toggling, and clean server shutdown (`shutdownServers`).
- **Autonomous Self-Healing Loop:** Scrapes web pages to a structural JSON AST, generates native Rust UI code, and iterates on compilation checks.

## Quickstart

### Build and Run
```bash
# Build release binary
cargo build --release

# Run with virtual display (headless Linux)
xvfb-run -a cargo run --release

# Run in headless benchmark mode
BENCHMARK_MODE=1 cargo run --release
```

### End-to-End Test Suite
```bash
npm install
npm run benchmark
npm run test:e2e
```

### Python IPC Client
```python
from python.nativefy import NativeUI

ui = NativeUI()
ui.toggle_dashboard()
ui.screenshot("output.png")
ui.shutdown()
```
