const fs = require('fs');
const { execSync } = require('child_process');

const PERF_FILE = 'perf_metrics.json';
const HISTORY_LIMIT = 10;
const THRESHOLDS = {
    layout_time_micros: 5000, // 5ms
    frame_time_micros: 33333, // 30 FPS min
};

const history = [];

function getScalingCommand(metrics) {
    history.push(metrics);
    if (history.length > HISTORY_LIMIT) history.shift();

    if (history.length < 5) return null;

    const avgLayout = history.reduce((sum, m) => sum + m.layout_time_micros, 0) / history.length;
    const avgFrame = history.reduce((sum, m) => sum + m.frame_time_micros, 0) / history.length;

    console.log(`[Health] Avg Layout: ${avgLayout.toFixed(2)}µs | Avg Frame: ${avgFrame.toFixed(2)}µs`);

    let batchSize = 100;
    let textThreshold = 200;
    let textureThreshold = 50;

    if (avgLayout < 1000 && avgFrame < 10000) {
        // High performance headroom: Aggressive scaling
        console.log("[Health] Performance headroom detected. Scaling UP resources.");
        batchSize = 500;
        textThreshold = 1000;
        textureThreshold = 200;
    } else if (avgLayout > 4000 || avgFrame > 25000) {
        // Pressure detected: Tighten resources
        console.log("[Health] Resource pressure detected. Scaling DOWN resources.");
        batchSize = 50;
        textThreshold = 100;
        textureThreshold = 20;
    }

    // This script doesn't have direct bridge access, it would normally write a command file
    // or trigger an engine-exposed CLI. Since we are in a simulation, we'll log the intended command.
    return `NativeUI.scaleResources(${batchSize}, ${textThreshold}, ${textureThreshold})`;
}

function monitorHealth() {
    console.log("Starting Continuous Health Monitor with Auto-Scaling...");

    setInterval(() => {
        if (fs.existsSync(PERF_FILE)) {
            try {
                const metrics = JSON.parse(fs.readFileSync(PERF_FILE, 'utf8'));
                const scaleCmd = getScalingCommand(metrics);

                if (scaleCmd) {
                    console.log(`[Health] Intended Scale Command: ${scaleCmd}`);
                    // In the real system, this is wired via the JS runtime's internal scheduler.
                }

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
            } catch (e) {
                console.error("[Health] Error reading metrics.");
            }
        } else {
            console.log("[Health] Waiting for engine to export metrics...");
        }
    }, 5000);
}

monitorHealth();
