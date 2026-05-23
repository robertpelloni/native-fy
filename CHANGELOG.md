# CHANGELOG

## [0.30.0] - 2026-05-23
- Implemented Dynamic Resource Auto-Scaling module in the core engine.
- Added `ScaleResources` command to dynamically adjust command batch sizes and cache thresholds.
- Enhanced the Autonomous Scheduler (JS) and Health Monitor with scaling intelligence.
- Wired `NativeUI.scaleResources` bridge for runtime resource orchestration.

## [0.29.0] - 2026-05-23
- Implemented functional WGPU-to-CPU buffer mapping for high-fidelity screenshots.
- Refactored texture and text buffer caches to use an LRU eviction strategy for improved memory stability.
- Integrated an autonomous task scheduler into the JS runtime for continuous maintenance.
- Initiated the "Live Autonomous Protocol Deployment" for long-term stress testing.
- Wired advanced bridge features (`fetch`, `reload`) to the default UI tree for verification.

## [0.28.1] - 2026-05-23
- Integrated core system integration tests into the build workflow (`build.rs`).
- Implemented a Throughput Stress Test (`scripts/stress_test.js`) for high node counts.
- Validated engine stability with 5000+ nodes and meeting latency targets.
- Refined automated repository synchronization and intelligent merge protocols.

## [0.27.0] - 2026-05-23
- Implemented core system integration testing suite (`scripts/integration_test.js`).
- Unified the autonomous lifecycle under a single orchestration command.
- Verified bridge interface consistency across JS and Rust layers.
- Formally integrated core validation as a mandatory E2E lifecycle stage.

## [0.26.0] - 2026-05-23
- Implemented a runtime toggle for Dashboard Mode (`toggleDashboard`).
- Enhanced the Native Monitoring Dashboard with real-time performance history graphs.
- Integrated the dashboard rendering path with the high-performance text engine.
- Refined the graph visualization for better visibility in production environments.
