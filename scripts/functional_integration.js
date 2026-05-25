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

    // 2. Headless Verification & Sustained Load Pass
    try {
        console.log("\nStage 2: Verifying sustained load pass (3 iterations)...");
        const binaryName = process.platform === 'win32' ? 'app.exe' : './app';
        const binaryPath = path.join(STAGING_DIR, binaryName);

        const iterations = 3;
        let totalLayoutTime = 0;

        for (let i = 1; i <= iterations; i++) {
            console.log(`Iteration ${i}/${iterations}...`);
            // Execute staging binary in benchmark mode
            execSync(`BENCHMARK_MODE=1 ./${binaryName}`, {
                cwd: STAGING_DIR,
                stdio: 'pipe'
            });

            const metricsPath = path.join(STAGING_DIR, PERF_FILE);
            if (fs.existsSync(metricsPath)) {
                const metrics = JSON.parse(fs.readFileSync(metricsPath, 'utf8'));
                totalLayoutTime += metrics.layout_time_micros;
                console.log(`[OK] Iteration ${i} complete: ${metrics.layout_time_micros}µs`);
            } else {
                throw new Error("perf_metrics.json was not generated in staging.");
            }
        }

        console.log(`[OK] Sustained load verification complete. Average Layout: ${Math.round(totalLayoutTime / iterations)}µs`);
    } catch (e) {
        console.error("Functional verification FAILED:", e.message);
        process.exit(1);
    }

    console.log("\n[Success] Target Environment Functional Integration PASSED.");
}

runFunctionalIntegration();
