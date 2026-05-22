# VISION: Native-fy

## Core Philosophy
Native-fy is an ultra-lightweight UI engine designed to bypass the overhead of modern web engines (Chromium, WebKit). It aims to provide the developer experience of the web (HTML/CSS-like layout and JavaScript logic) with the performance and efficiency of native Rust.

## Ultimate Goal
A single binary under 10MB that can render complex, interactive UIs at 60+ FPS with minimal CPU/RAM usage, suitable for embedded devices and high-performance desktop apps.

## Key Foundational Concepts
1. **No Web Engine:** No DOM, no CSS parser (initially), no Blink/Gecko.
2. **Native Layout:** Powered by Taffy (Rust implementation of Flexbox).
3. **GPU Rendering:** Hardware-accelerated quads and text via wgpu and glyphon.
4. **QuickJS Logic:** Low-overhead JavaScript runtime for business logic and UI reactivity.
5. **AI-First Compilation:** Use LLMs to "compile" web structural descriptions (JSON AST) into optimized native Rust code, bypassing the need for a complex runtime layout engine where possible.

## User-Satisfaction Design
- **Instant Start:** Sub-100ms cold start to interactive.
- **Fluidity:** Zero-jank animations and scrolling.
- **Portability:** Compiles to a single static binary.
