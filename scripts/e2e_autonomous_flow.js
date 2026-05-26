const { execSync, exec } = require('child_process');
const fs = require('fs');

const DURATION_MS = 60000; // 1 minute simulation
const E2E_LOG = 'e2e_autonomous.log';

function log(message) {
    fs.appendFileSync(E2E_LOG, `[${new Date().toISOString()}] ${message}\n`);
    console.log(message);
}

function runStage(name, command) {
    log(`\n--- [Stage] ${name} ---`);
    try {
        execSync(command, { stdio: 'inherit' });
        log(`[Success] Stage: ${name}`);
    } catch (e) {
        log(`[Failure] Stage: ${name}`);
        process.exit(1);
    }
}

async function runE2EAutonomousFlow() {
    if (fs.existsSync(E2E_LOG)) fs.unlinkSync(E2E_LOG);

    log("Starting Full End-to-End Autonomous Lifecycle Test...");

    runStage("Protocol Sync", "npm run protocol-sync");
    runStage("Cargo Build", "cargo build --release");

    log("\n--- [Stage] Live Autonomous Execution Simulation (1m) ---");
    // Start engine with validation mode for faster scheduler heartbeats
    const child = exec('VALIDATION_MODE=1 PROD_MODE=1 xvfb-run -a cargo run --release');

    let scalingEvents = 0;
    let schedulerEvents = 0;

    child.stdout.on('data', (data) => {
        if (data.includes('Runtime: Scaling resources')) {
            scalingEvents++;
            log(`[Engine] Scaling Event: ${data.trim()}`);
        }
        if (data.includes('Scheduler: Running autonomous maintenance pass')) {
            schedulerEvents++;
            log(`[Engine] Scheduler Event: ${data.trim()}`);
        }
    });

    child.stderr.on('data', (data) => {
        if (data.includes('ERROR')) {
            log(`[Error] ${data.trim()}`);
        }
    });

    return new Promise((resolve) => {
        setTimeout(() => {
            log("\n## E2E Validation Summary");
            log(`- Duration: ${DURATION_MS / 1000}s`);
            log(`- Scaling Decisions: ${scalingEvents}`);
            log(`- Maintenance Cycles: ${schedulerEvents}`);

            if (scalingEvents > 0 && schedulerEvents > 0) {
                log("[PASS] Full autonomous integration verified.");
            } else {
                log("[FAIL] Incomplete autonomous activity detected.");
                log(`Check log: ${E2E_LOG}`);
            }

            child.kill();
            resolve();
        }, DURATION_MS);
    });
}

runE2EAutonomousFlow().then(() => {
    console.log("\nE2E Autonomous Flow Validation Complete.");
    process.exit(0);
});
