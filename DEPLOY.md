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

## Usage
1. **Extract AST:**
   ```bash
   node scripts/web_scraper.js "https://example.com" > ui_ast.json
   ```

2. **Compile Native UI:**
   ```bash
   node scripts/compiler_agent.js ui_ast.json
   ```
   *This will recursively prompt the LLM and run `cargo check` until the output successfully builds.*

## Building & Running
- **Debug:** `cargo run`
- **Release:** `cargo build --release` (binary at `target/release/app`)

## Headless Execution
On servers without a GPU/Display:
1. Use `XVFB` or similar if windowing is required.
2. Monitor `app.log` for output as no window will be visible.
3. Rendering may fail unless a software Vulkan driver (like `lavapipe`) is present.

## Monitoring Stability
To run the application with live performance monitoring:
```bash
node scripts/monitor.js
```
*This will log system resources and engine metrics to `stability.log`.*

## Full Validation Pipeline
To run the full E2E autonomous pipeline (Extraction -> Compilation -> Test -> Benchmark -> Deploy -> E2E Validation):
```bash
npm run pipeline
```
