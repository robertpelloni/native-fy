# IDEAS

## Feature Expansions
- **Hot Reloading:** Implement a file watcher that triggers the `compiler_agent.js` and reloads the native window without a full restart.
- **SDF UI Components:** Use Signed Distance Fields (SDFs) for rendering perfectly anti-aliased rounded corners, shadows, and text at any scale.
- **WebAssembly Support:** Allow QuickJS to call into WASM modules for high-performance JS-side logic.
- **Vulkan/Metal Direct Backends:** While `wgpu` handles this, providing direct backend optimizations for specific OSs could improve latency.

## Architectural Pivots
- **Hybrid Rendering:** Explore using `piet-gpu` or `vello` for 2D vector graphics if `wgpu` primitive drawing becomes too complex.
- **Unified AST:** Standardize the AST further so it can be exported to other native engines (e.g., Swift UI or Jetpack Compose) as a secondary target.
