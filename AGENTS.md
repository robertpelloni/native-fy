# Project Architecture: AI Native UI Engine

This project builds an ultra-lightweight desktop runtime that bypasses Chromium by compiling layout trees into a native Rust GPU-rendered scene graph.

## Strict Technology Stack Constraints
1. **Rust Core:** `winit` for windowing, `wgpu` for hardware-accelerated rendering, and `taffy` for layout computation.
2. **JavaScript Engine:** `QuickJS` via Rust bindings to run application business logic.
3. **No Web Engines:** Absolute ban on Chromium, WebKit, or Edge WebView2. Do not use standard HTML DOM elements or CSS properties outside of the Taffy specification.

## Core Translation Abstractions
All layouts must compile strictly down to a structural JSON AST matching this shape:
- Primitives allowed: `Box`, `Text`, `Image`, `Input`, `List`
- Styling allowed: Flexbox properties only (via Taffy mapping: `flex-direction`, `padding`, `margin`, `align-items`, `justify-content`).

## Jules Operational Objective
Your job is to build the continuous self-healing loop:
1. Parse a structural JSON AST generated from a web scraping pass.
2. Use an internal code-generation loop to translate that AST into the native `wgpu` + `taffy` Rust UI tree.
3. Compile and test the output in your cloud VM. If compilation fails, analyze the compiler logs and execute a self-correcting rewrite pass.