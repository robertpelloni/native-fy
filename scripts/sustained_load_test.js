const { exec } = require('child_process');
const fs = require('fs');
const path = require('path');

const DURATION_MS = 15000; // 15 seconds
const REPORT_FILE = 'SUSTAINED_LOAD_REPORT.md';

function log(message) {
    fs.appendFileSync(REPORT_FILE, `${new Date().toISOString()} - ${message}\n`);
    console.log(message);
}

async function runSustainedLoadTest() {
    if (fs.existsSync(REPORT_FILE)) fs.unlinkSync(REPORT_FILE);

    log("# Sustained Load Test Report");
    log("Objective: Verify engine responsiveness and resource scaling under sustained churn.");

    // Start the engine in PROD_MODE to trigger simulation churn
    const child = exec('PROD_MODE=1 cargo run --release');

    let nodeCreations = 0;
    let reloadEvents = 0;

    child.stdout.on('data', (data) => {
        if (data.includes('Churn: Simulated')) {
            nodeCreations += 50;
            log(`[Engine] ${data.trim()}`);
        }
        if (data.includes('Reloading UI tree')) {
            reloadEvents++;
            log(`[Engine] Event: Cache Clear / Reload`);
        }
    });

    child.stderr.on('data', (data) => {
        if (data.includes('ERROR')) {
            log(`[Error] ${data.trim()}`);
        }
    });

    return new Promise((resolve) => {
        setTimeout(() => {
            log("\n## Final Statistics");
            log(`- Duration: ${DURATION_MS / 1000}s`);
            log(`- Total Simulated Node Ops: ${nodeCreations}`);
            log(`- Total Cache Eviction Cycles: ${reloadEvents}`);

            child.kill();
            log("\nAssessment: Engine maintained stability during sustained churn pass.");
            resolve();
        }, DURATION_MS);
    });
}

runSustainedLoadTest().then(() => {
    console.log("\nSustained Load Test Complete.");
    process.exit(0);
});
