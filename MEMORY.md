# MEMORY

## Architectural Decisions

- **Winit ApplicationHandler:** Using the `ApplicationHandler` pattern introduced in `winit` 0.30 for cleaner lifecycle management and future-proofing.
- **Wgpu Integration:** Chose `wgpu` over lower-level APIs for cross-platform GPU abstraction. All UI elements will eventually be drawn as custom primitives (SDF-based or traditional triangles).
- **Taffy for Layout:** Standardized on Taffy for its CSS Flexbox compliance without the bloat of Yoga or other C++-based engines.
- **Self-Healing Loop:** Established an LLM-driven remediation loop in `scripts/compiler_agent.js` to handle the complexity of mapping dynamic web ASTs to rigid Rust types.

## Known Constraints

- **No DOM:** Absolute strictness on bypassing web engines. All UI must be native.
- **Flexbox Only:** Layout is restricted to Taffy's supported Flexbox subset.
