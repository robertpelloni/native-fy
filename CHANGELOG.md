# CHANGELOG

## [0.38.0] - $(date +%Y-%m-%d)
- Automated synchronization from upstream.
- Conducted intelligent merge resolution on active feature branches.

$(cat CHANGELOG.md | grep -v "# CHANGELOG")
- Implemented Python language bindings using `pyo3` allowing developers to execute Python logic scripts that interface with the `NativeUI` Rust bridge.
- Refined the Event Bridging layer by implementing precise tree-based `hit_test` logic to map cursor and click events strictly to target nodes rather than generic window coordinates.
- Enhanced Autonomous Task Scheduler: The JavaScript scheduler now dynamically triggers a full `RunPipeline` recovery mechanism if performance metrics stall (FPS < 5) to ensure runtime stability and self-healing.
- **Watchdog Execution Protocol:** Implemented the `RunPipeline` UI Command to execute the Node.js End-to-End (`test:e2e`) pipeline externally. This enables the engine to completely test and recover itself when the JavaScript scheduler detects fatal performance degradation.
- **System-Aware Resource Orchestration:** Upgraded the `src/monitor.rs` loop to actively monitor the host machine's total RAM usage alongside the application's RSS memory fingerprint. When memory pressure exceeds 85%, the engine aggressively scales down its internal cache structures (Text, Textures) and shrinks render batch sizes to prevent OOM termination.
- **Core System Integration & Lifecycle Validation:** Implemented an automated E2E lifecycle verification sequence during engine boot in `src/runtime.js`.
