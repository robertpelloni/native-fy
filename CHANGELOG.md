# CHANGELOG

## [0.22.0] - 2026-05-23
- Implemented Full Automated E2E Lifecycle validation (`scripts/e2e_test.js`).
- Integrated `PROD_MODE` to silence debug overlays for clean production deployment.
- Enhanced `BENCHMARK_MODE` for immediate metric export in headless environments.
- Formally established the "start" and "test:e2e" production scripts in `package.json`.

## [0.21.0] - 2026-05-23
- Implemented Native Monitoring Dashboard mode (`DASHBOARD_MODE`).
- Integrated automated runtime health checks and heartbeat mechanisms.
- Created `scripts/health_monitor.js` watchdog for production validation.
- Established `HEALTH.md` governance for monitoring protocols.

## [0.20.0] - 2026-05-23
- Integrated automated performance benchmarking into the pipeline.
- Implemented `perf_metrics.json` export in the native engine.
- Created `scripts/benchmark_runner.js` for performance validation and reporting.
- Updated `PERFORMANCE.md` with automated benchmark tracking.
