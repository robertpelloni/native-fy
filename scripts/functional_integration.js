const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const STAGING_DIR = 'staging';
const PERF_FILE = 'perf_metrics.json';

async function runFunctionalIntegration() {
    console.log("--- Starting Target Environment Functional Integration ---");

    // 1. Promote latest build
    try {
        console.log("Stage 1: Promoting build to staging...");
        execSync('npm run deploy:staging', { stdio: 'inherit' });
    } catch (e) {
        console.error("Promotion failed.");
        process.exit(1);
    }

    // 2. Headless Verification
    try {
        console.log("\nStage 2: Verifying headless layout pass...");
        const binaryName = process.platform === 'win32' ? 'app.exe' : './app';
        const binaryPath = path.join(STAGING_DIR, binaryName);

        // Execute staging binary in benchmark mode
        execSync(`BENCHMARK_MODE=1 ${binaryPath}`, {
            cwd: STAGING_DIR,
            stdio: 'inherit'
        });

        const metricsPath = path.join(STAGING_DIR, PERF_FILE);
        if (fs.existsSync(metricsPath)) {
            const metrics = JSON.parse(fs.readFileSync(metricsPath, 'utf8'));
            console.log(`[OK] Layout verification complete. Timings: ${metrics.layout_time_micros}µs`);
        } else {
            throw new Error("perf_metrics.json was not generated in staging.");
        }
    } catch (e) {
        console.error("Functional verification FAILED:", e.message);
        process.exit(1);
    }

    console.log("\n[Success] Target Environment Functional Integration PASSED.");
}

runFunctionalIntegration();
