# DEPLOY

## Prerequisites

- Node.js (v18+ recommended for native fetch)
- Rust toolchain (`cargo`)
- Playwright system dependencies (`npx playwright install`)

## Setup

1. **Install Node Dependencies:**
   ```bash
   npm install
   npx playwright install
   ```

2. **Environment Variables:**
   Copy the example environment file:
   ```bash
   cp .env.example .env
   ```
   Open `.env` and fill in your `GEMINI_API_KEY`. This is strictly required for Phase 3 (Compiler & Self-Healing Engine) to interact with the LLM.

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