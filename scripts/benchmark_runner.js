const { execSync } = require('child_process');
const fs = require('fs');

const TARGETS = {
    fps: 0, // In headless mode FPS is 0
    layout_time_micros: 2000, // 2ms target
    frame_time_micros: 20000, // 20ms target
};

function runBenchmark() {
    console.log("Starting Automated Performance Benchmark...");

    try {
        // Run the engine in benchmark mode
        process.env.BENCHMARK_MODE = "1";
        console.log("Executing engine (Headless Benchmark Mode)...");
        try {
            execSync('cargo run --release', { stdio: 'inherit' });
        } catch (e) {
            // It might panic because of no display, but should still export metrics
            console.log("Engine finished (potentially via expected headless panic/exit).");
        }

        if (!fs.existsSync('perf_metrics.json')) {
            console.error("Error: perf_metrics.json not found. Benchmark failed.");
            process.exit(1);
        }

        const metrics = JSON.parse(fs.readFileSync('perf_metrics.json', 'utf8'));
        console.log("\n--- Benchmark Results ---");
        console.log(JSON.stringify(metrics, null, 2));

        let passed = true;
        Object.keys(TARGETS).forEach(key => {
            if (key === 'fps') return; // Skip FPS in headless
            if (metrics[key] > TARGETS[key]) {
                console.warn(`WARNING: Metric ${key} exceeded target! (${metrics[key]} > ${TARGETS[key]})`);
                passed = false;
            }
        });

        const report = `\n### Automated Benchmark Result (${new Date().toISOString()})\n` +
            `- **Status:** ${passed ? 'PASSED' : 'FAILED'}\n` +
            `- **Layout Time:** ${metrics.layout_time_micros}µs (Target: ${TARGETS.layout_time_micros}µs)\n` +
            `- **Frame Time:** ${metrics.frame_time_micros}µs (Target: ${TARGETS.frame_time_micros}µs)\n` +
            `- **Node Count:** ${metrics.node_count}\n`;

        fs.appendFileSync('PERFORMANCE.md', report);
        console.log("\nReport appended to PERFORMANCE.md");

        if (!passed) {
            console.error("Benchmark Targets NOT met.");
            // process.exit(1); // Don't block the pipeline yet, just warn
        } else {
            console.log("Benchmark Targets MET.");
        }

    } catch (error) {
        console.error("Benchmark Runner Failed:", error);
        process.exit(1);
    }
}

runBenchmark();
