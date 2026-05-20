# Project Architecture & Decisions Summary: Native-fy (AI Native UI Engine)

## Overview & Vision
The "Native-fy" project aims to build an ultra-lightweight desktop UI runtime that bypasses heavy web engines entirely (no Chromium, WebKit, or WebView2). It achieves this by extracting structural web layout trees into a normalized JSON AST, and then compiling that AST down to a native, GPU-rendered scene graph using Rust. It uses an AI-driven, continuous "self-healing" compilation loop to generate the native UI.

## Technology Stack
The stack is strictly defined and strictly constrained.
*   **Rust Core (Rendering & Windowing):**
    *   `winit`: For window creation and event loop management (`src/main.rs`).
    *   `wgpu`: For hardware-accelerated GPU rendering.
    *   `taffy`: For layout computation.
*   **JavaScript Engine (Business Logic):**
    *   `QuickJS`: Integrated via Rust bindings to handle the application's business logic layer (`src/runtime.js`).
*   **Web Scraping (Phase 1):**
    *   `Node.js` + `Playwright` + JavaScript: Used exclusively for extracting layout topology from existing web pages (`scripts/web_scraper.js`). We specifically opted for vanilla JS here instead of TypeScript to avoid compiling steps via `tsc` or external loaders.
*   **AI Code Generation:**
    *   Gemini API (via automated agent scripts and prompts like `prompts/transpiler_agent.txt`).

## Architecture Phases & Pipeline
The translation pipeline is divided into three distinct phases:
1.  **Phase 1: Structural AST Extraction (Completed)**
    *   A Playwright script (`scripts/web_scraper.js`) navigates to a given URL and strips away web-specific clutter.
    *   It executes a `page.evaluate` traversal algorithm that outputs a clean, normalized JSON Abstract Syntax Tree (AST) representing only the essential layout topology (boxes, coordinates, text, inputs, flexbox styles).
2.  **Phase 2: Rust Core Runtime Utilities (Completed)**
    *   Rust data structures (`src/layout.rs`) map the extracted JSON AST directly to `taffy` flexbox nodes.
    *   Strict type safety ensures that any unsupported styling properties throw clear validation errors.
3.  **Phase 3: Compiler & Self-Healing Engine (Next Priority - Blocked)**
    *   An LLM generation loop (Jules/Gemini) translates the JSON AST into native `wgpu` + `taffy` code.
    *   **Self-Healing Loop:** The generated code is compiled locally using `cargo check` and `cargo test`. If it fails, the compiler's `stdout`/`stderr` traces are fed back to the LLM to recursively patch the source files until the project builds successfully.
    *   **Status: Blocked.** Implementing this phase requires calling the Gemini API, which strictly requires an API key (secrets). In accordance with the project instructions, I am stopping here since secrets are required and cannot be safely hardcoded or executed without external credentials.

## Core Translation Abstractions & Constraints
*   **Strict Web Ban:** Standard HTML DOM elements and CSS properties are strictly forbidden outside of the Taffy specification.
*   **Allowed Primitives:** The UI is built entirely out of: `Box`, `Text`, `Image`, `Input`, and `List`.
*   **Allowed Styling:** Only Flexbox properties supported by `taffy` are permitted (e.g., `flex-direction`, `padding`, `margin`, `align-items`, `justify-content`).

## Development Patterns & Conventions
*   **Documentation-Driven Agents:** The project uses an `AGENTS.md` file as the absolute source of truth for architectural constraints. Other model-specific instructions (`CLAUDE.md`, `GEMINI.md`, etc.) reference this root file to prevent hallucination.
*   **Project Management Docs:** Comprehensive status and planning are maintained across `HANDOFF.md`, `ROADMAP.md`, `TODO.md`, `VISION.md`, `CHANGELOG.md`, and `DEPLOY.md`.
*   **Versioning:** `VERSION.md` serves as the single source of truth for the project version to avoid hardcoded version strings across the app. All environments (`Cargo.toml`, `package.json`, etc.) must stay synchronized with this file.
*   **Coding Standards:**
    *   No hardcoding, printing, logging, or committing of API keys/secrets. Use `.env.example` placeholders.
    *   Comments are strictly reserved for non-obvious logic, side effects, tradeoffs, or known limitations (no obvious comments).
    *   Refactoring is only permitted if it clearly simplifies code or removes redundancy without changing behavior.
    *   All code generation must be verified against `cargo check/test` or Node test verifications before being considered complete.
    *   If the next task requires secrets (like Phase 3 requiring Gemini API keys), execution stops to prevent accidental exposure or blockage.