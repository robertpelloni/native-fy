const { exec } = require('child_process');
const fs = require('fs');

const LIVE_LOG = 'live_deployment.log';
const DURATION_MS = 300000; // 5 minutes

function log(message) {
    const entry = `[${new Date().toISOString()}] ${message}\n`;
    fs.appendFileSync(LIVE_LOG, entry);
    console.log(message);
}

log("--- Starting Live Autonomous Protocol Deployment Simulation ---");

// Start the engine in production mode
const child = exec('PROD_MODE=1 cargo run --release');

child.stdout.on('data', (data) => {
    if (data.includes('Health Check:') || data.includes('Memory:')) {
        log(data.trim());
    }
});

child.stderr.on('data', (data) => {
    log(`ENGINE ERROR: ${data.trim()}`);
});

// Periodic external interaction simulation
const interactionInterval = setInterval(() => {
    log("Simulation: Triggering environment sync...");
    try {
        require('child_process').execSync('npm run protocol-sync');
    } catch (e) {}
}, 60000);

setTimeout(() => {
    log("Live simulation complete. Terminating process...");
    clearInterval(interactionInterval);
    child.kill();

    // Summary
    const stats = fs.statSync(LIVE_LOG);
    log(`Simulation finished. Log size: ${stats.size} bytes.`);
    process.exit(0);
}, DURATION_MS);
