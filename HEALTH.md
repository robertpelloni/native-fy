# HEALTH: Native-fy Runtime Monitoring

## Monitoring Dashboards
The engine supports a dedicated `DASHBOARD_MODE`. When the `DASHBOARD_MODE` environment variable is set, the engine renders historical performance data and status indicators instead of the standard UI.

## Automated Health Checks
- **Bridge Heartbeat:** The JS runtime periodically calls `NativeUI.healthCheck()` to ensure the message bridge is responsive.
- **Metric Export:** The engine exports `perf_metrics.json` which includes FPS, layout latency, and frame timing.
- **External Watchdog:** `scripts/health_monitor.js` tracks exported metrics and triggers automated recovery (e.g., repository sync) if thresholds are exceeded.

## Recovery Protocols
If the engine becomes unresponsive or performance degrades:
1. **Watchdog Alert:** Triggered by threshold violations.
2. **Auto-Sync:** Triggers `npm run protocol-sync` to re-validate documentation and environment state.
3. **Log Dump:** Errors are persisted to `app.log` for subsequent AI agent analysis.
