# IDEAS: Native-fy Expansion & Pivots

## Feature Expansions
- **Hot Reloading:** Inject updated QuickJS scripts without restarting the Rust process.
- **Component Library:** Build a set of standard UI components (Buttons, Inputs, Modals) in JS.
- **Accessibility:** Implement a native accessibility tree bridge for screen readers.

## Structural Refactoring
- **WASM Backend:** Port the core engine to WASM to run in the browser as a "meta-engine" (paradoxical but interesting for demos).
- **Python Bindings:** Allow using Python instead of QuickJS for logic.
- **Zig Port:** Port the rendering core to Zig for even lower binary size and faster build times.

## Pivots
- **Game Engine Overlay:** Position Native-fy as a lightweight UI overlay for game engines (Godot/Bevy).
- **Embedded OS UI:** Target bare-metal or ultra-light Linux for smart appliances.
- **Secure Browser:** Build a specialized, limited-capability browser that only renders trusted AST layouts for high-security environments.
