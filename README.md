# Native-fy

An ultra-lightweight desktop runtime that bypasses Chromium by compiling layout trees into a native Rust GPU-rendered scene graph using wgpu, Taffy, and QuickJS.

## Overview
This project builds an autonomous AI-native UI engine that extracts structural layout representations from the web and compiles them into a native desktop shell entirely bypassing web engines like Chromium or WebKit.

## Core Features
- **GPU Accelerated Rendering:** Uses `wgpu` to drive an instanced rendering pipeline capable of hardware-accelerated 60 FPS performance.
- **Native Layout:** Implements `taffy` (flexbox) for exact structural mapping.
- **QuickJS Scripting Bridge:** Background worker thread isolates Javascript application logic, communicating over MPSC channels.
- **Dynamic Resource Orchestration:** Wgpu global telemetry handles real-time scaling of LRU caches to prevent memory bloat.
- **Vector Graphics:** Native path tessellation mapping via `usvg`/`resvg` allowing `NativeUI.createSvg()`.

## Installation & Deployment
Refer to [DEPLOY.md](DEPLOY.md) for full setup instructions, including the autonomous testing validation pipeline (`npm run pipeline`).

## Architecture Constraints
See [AGENTS.md](AGENTS.md) for strict architectural rules. No standard HTML DOM elements are allowed, everything is mapped directly to `AstRect` UI Node primitives.
