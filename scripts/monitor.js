const { exec } = require('child_process');
const fs = require('fs');

const LOG_FILE = 'stability.log';
const DURATION_MS = 30000; // 30 seconds

function log(message) {
    const entry = `[${new Date().toISOString()}] ${message}\n`;
    fs.appendFileSync(LOG_FILE, entry);
    console.log(message);
}

log("Starting stability monitoring session...");

const child = exec('cargo run --release');

child.stdout.on('data', (data) => {
    if (data.includes('Performance:')) {
        log(data.trim());
    }
});

child.stderr.on('data', (data) => {
    log(`ERROR: ${data.trim()}`);
});

setTimeout(() => {
    log("Monitoring session complete. Terminating process...");
    child.kill();
    process.exit(0);
}, DURATION_MS);
