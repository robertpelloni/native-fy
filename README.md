# Native-fy

**Current Version:** v0.40.0

Native-fy is an ultra-lightweight UI engine designed to completely bypass the overhead of modern web engines (Chromium, WebKit). Its primary goal is to deliver the familiar developer experience of the web (HTML/CSS-like structural layouts and JavaScript/Python scripting logic) but execute it with the raw performance, memory efficiency, and minimal footprint of native Rust code.

The ultimate target is a single binary under 10MB capable of rendering complex, interactive UIs at 60+ FPS with minimal CPU and RAM usage, making it ideal for embedded devices and high-performance desktop applications.

## Technical Architecture & Stack
The project enforces strict boundaries and dependencies to achieve its performance goals:

*   **Rust Core (The Shell):**
    *   **Windowing & OS Integration:** Handled via `winit`.
    *   **Hardware-Accelerated Rendering:** Powered by `wgpu`. The pipeline uses instanced quad rendering for maximum efficiency in UI layouts.
    *   **Layout Engine:** Employs `taffy` (a Rust implementation of Flexbox/Grid) to map web-like structures without the bloat of a full CSS parser or DOM.
    *   **Text Rendering:** Uses `glyphon` for high-performance, native font rendering.
    *   **Image & Vector Support:** Images are loaded asynchronously; SVG/Vector graphics are supported via `resvg` and `tiny-skia`.
*   **Scripting Layers (The Logic):**
    *   **JavaScript Engine:** `QuickJS` via `rquickjs` bindings. This provides a lightweight JS environment strictly for application logic, state management, and UI reactivity.
    *   **Python Engine:** `pyo3` integration allows Python scripts to execute logic and interact with the Native UI bridge via the Free-threaded Python GIL.
    *   **Bridge (Rust <-> Scripts):** A robust, asynchronous bridge facilitates communication between the runtime engines and the native shell.
    *   **Communication Protocol:** Communication is handled via an asynchronous MPSC channel using a strongly typed `UiCommand` enum (e.g., `CreateNode`, `UpdateText`, `Fetch`, `HotReloadScript`). This ensures explicit, batched, and thread-safe interactions.
*   **No Web Engines allowed:** The project strictly prohibits Chromium, WebKit, Edge WebView2, standard HTML DOM elements, and CSS properties outside the supported Taffy flexbox subset.

## Automated Translation Pipeline (The "Transpiler")
A key feature of Native-fy is its AI-assisted compilation pipeline:
1.  **AST Extraction:** A Playwright script (`scripts/web_scraper.js`) navigates to a URL and extracts a clean, normalized JSON Abstract Syntax Tree (AST) representing only essential structural boxes, flexbox properties, and content.
2.  **Compilation Loop:** A Node.js script (`scripts/compiler_agent.js`) feeds this AST to an LLM (using Gemini API).
3.  **Code Generation & Self-Healing:** The LLM generates the corresponding native Rust UI tree code. Crucially, this output is immediately tested (`cargo check`). If compilation fails, the compiler trace is fed back into the LLM in a self-correcting loop until a successfully building artifact is produced.

## Key Design Decisions & Patterns
*   **Memory Safety & Stability:** Resources (textures, text buffers) are managed aggressively using LRU (Least Recently Used) cache eviction policies to prevent memory leaks during long sessions.
*   **Hot-Reloading:** The system completely drops and recreates the `JsRuntime` instance on script reload, rather than simply re-evaluating the script in the existing context, to prevent memory leaks and duplicate event listeners.
*   **Asynchronous Operations:** Asset loading (images, network requests via a polyfilled `fetch`) is strictly asynchronous to prevent blocking the main rendering thread and maintain 60 FPS.
*   **Performance Monitoring:** The engine includes built-in instrumentation (e.g., Native Monitoring Dashboard) for real-time tracking of loop latency, resource usage, and bridge throughput.
*   **Autonomous Operation:** The system features background JS tasks (Watchdogs, Schedulers) and Auto-Scaling logic that dynamically adjusts cache sizes and batch limits based on host OS metrics (via `sysinfo`).

## Quick Start
Ensure you have Rust, Node.js, and Playwright dependencies installed.

```bash
# Clone the repository
git clone https://github.com/robertpelloni/native-fy.git
cd native-fy

# Run E2E verification
npm install
npx playwright install
npm run test:e2e

# Run the core UI engine in release mode
cargo run --release
```

## E2E Validation and Telemetry
The core runtime includes an automated E2E lifecycle verification sequence during engine boot. You can manually inspect the engine's internal latency and scaling bounds by enabling the dashboard overlay:

* Click "Toggle Dashboard" inside the Native-fy window or trigger `NativeUI.toggleDashboard()` via the JS bridge.
