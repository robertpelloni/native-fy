# DEPLOY: Deployment & Setup

## Prerequisites
- Rust (latest stable)
- Cargo
- Node.js (v18+ recommended for native fetch)
- Playwright system dependencies (`npx playwright install`)
- Vulkan/Metal/DX12 compatible hardware (or software rasterizer for headless)

## Setup
1. **Clone & Install Node Dependencies:**
   ```bash
   git clone <repo_url>
   cd native-fy
   npm install
   npx playwright install
   ```

2. **Environment Variables:**
   Copy the example environment file and fill in your `GEMINI_API_KEY`:
   ```bash
   cp .env.example .env
   ```

## Production Monitoring Dashboard Quickstart
The Native-fy Monitoring Dashboard is a key component for validating performance in production. It records telemetry, frame timings, and cache metrics.

To launch the dashboard overlay:
1. Ensure you are running in the target environment (local or XVFB).
2. Start the core UI engine:
   ```bash
   cargo run --release
   ```
3. Use `NativeUI.toggleDashboard()` or click the on-screen toggle to inspect memory bounds and loop latency in real-time.

For background telemetry logging (ideal for servers):
```bash
node scripts/monitor.js
```
*This logs system resources and engine metrics to `stability.log`.*

## Full Lifecycle Automation (E2E)
Validate the deployment and core integrations via the automated E2E tracking script:
```bash
npm run test:e2e
```
*This script will compile the SVG primitives, verify telemetry, test the bridge components, and check memory constraints without manual intervention.*

## Headless Execution
On servers without a GPU/Display:
1. Use `XVFB` or similar if windowing is required.
2. Monitor `app.log` for output as no window will be visible.
3. Rendering may fail unless a software Vulkan driver (like `lavapipe`) is present.
