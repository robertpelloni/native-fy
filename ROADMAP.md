# ROADMAP

## Phase 1: Structural AST Extraction
Implement a Playwright script (`scripts/web_scraper.ts`) that navigates to URLs and extracts layout topology, ignoring web clutter. It outputs a normalized JSON Abstract Syntax Tree (AST) focusing on structural boxes, coordinates, padding, margins, flexbox properties, text values, and inputs.

## Phase 2: Rust Core Runtime Utilities
Implement data structures in Rust (`src/layout.rs`) that directly map the extracted structural JSON AST to Taffy flexbox layout nodes, enforcing strict type safety to reject any properties outside the supported subset.

## Phase 3: Compiler & Self-Healing Engine
Create an automated agent script to handle the LLM generation loop, translating the JSON AST into the native `wgpu` + `taffy` Rust UI tree. This will involve an isolated execution loop running `cargo check/test` and feeding compiler errors back to the model for self-correction.