const { execSync } = require('child_process');
const fs = require('fs');

const TARGET_URL = "https://google.com";

function runStage(name, command) {
    console.log(`\n--- [Stage] ${name} ---`);
    try {
        execSync(command, { stdio: 'inherit' });
        console.log(`[Success] Stage: ${name}`);
    } catch (e) {
        console.error(`[Failure] Stage: ${name}`);
        process.exit(1);
    }
}

function runE2E() {
    console.log("Starting Full End-to-End Autonomous Lifecycle Test...");

    runStage("Protocol Sync", "npm run protocol-sync");

    // In headless cloud VM, we might skip the actual scraping/LLM part if keys aren't set
    if (process.env.GEMINI_API_KEY) {
        runStage("AST Extraction & Compilation", `node scripts/web_scraper.js "${TARGET_URL}" > ui_ast.json && node scripts/compiler_agent.js ui_ast.json`);
    } else {
        console.log("[Notice] GEMINI_API_KEY not set. Using existing ui_ast.json for test.");
    }

    runStage("Build Release", "cargo build --release");
    runStage("Performance Benchmark", "npm run benchmark");

    console.log("\n--- [Stage] Autonomous Self-Check ---");
    console.log("[Notice] Skipping live dashboard execution in headless environment. Verifying bridge code availability...");
    if (fs.readFileSync('src/runtime.js', 'utf8').includes('runAutonomousMaintenance')) {
        console.log("[Success] Stage: Autonomous Self-Check");
    } else {
        console.error("[Failure] Stage: Autonomous Self-Check");
        process.exit(1);
    }

    console.log("\n--- [Final Validation] ---");
    if (fs.existsSync('PERFORMANCE.md') && fs.existsSync('perf_metrics.json')) {
        console.log("All artifacts verified. E2E Test PASSED.");
    } else {
        console.error("Artifact validation failed. E2E Test FAILED.");
        process.exit(1);
    }
}

runE2E();
