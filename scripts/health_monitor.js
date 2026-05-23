const fs = require('fs');
const { execSync } = require('child_process');

const PERF_FILE = 'perf_metrics.json';
const THRESHOLDS = {
    layout_time_micros: 5000, // 5ms
    frame_time_micros: 33333, // 30 FPS min
};

function monitorHealth() {
    console.log("Starting Continuous Health Monitor...");

    setInterval(() => {
        if (fs.existsSync(PERF_FILE)) {
            const metrics = JSON.parse(fs.readFileSync(PERF_FILE, 'utf8'));
            console.log(`[Health] Current FPS: ${metrics.fps} | Layout: ${metrics.layout_time_micros}µs`);

            let unhealthy = false;
            Object.keys(THRESHOLDS).forEach(key => {
                if (metrics[key] > THRESHOLDS[key]) {
                    console.warn(`[Health] ALERT: Threshold exceeded for ${key}!`);
                    unhealthy = true;
                }
            });

            if (unhealthy) {
                console.log("[Health] Attempting automatic recovery (triggering sync)...");
                try {
                    execSync('npm run protocol-sync');
                } catch (e) {
                    console.error("[Health] Recovery failed.");
                }
            }
        } else {
            console.log("[Health] Waiting for engine to export metrics...");
        }
    }, 5000);
}

monitorHealth();
