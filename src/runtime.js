const _eventListeners = {};

const NativeUI = {
    createNode: (type, styles, text) => {
        return _native_create_node(type, styles, text);
    },
    setStyle: (nodeId, styles) => {
        return _native_set_style(nodeId, styles);
    },
    addEventListener: (type, callback) => {
        if (!_eventListeners[type]) {
            _eventListeners[type] = [];
        }
        _eventListeners[type].push(callback);
    },
    fetch: async (url) => {
        // Synchronous bridge call wrapped in an async JS interface
        const text = _native_fetch(url);
        return {
            text: async () => text,
            json: async () => JSON.parse(text)
        };
    },
    syncProtocol: () => {
        _native_sync_protocol();
    },
    getMetadata: () => {
        return _native_get_metadata();
    },
    getPerformanceStats: () => {
        return _native_get_perf_stats();
    },
    healthCheck: () => {
        _native_health_check();
    },
    reload: () => {
        _native_reload();
    },
    runPipeline: () => {
        _native_run_pipeline();
    },
    screenshot: (path) => {
        _native_screenshot(path);
    },
    toggleDashboard: () => {
        _native_toggle_dashboard();
    },
    scaleResources: (batchSize, textThreshold, textureThreshold) => {
        _native_scale_resources(batchSize, textThreshold, textureThreshold);
    },
    Components: {
        Button: (text, onClick, styles = {}) => {
            // Use native button implementation for better efficiency
            const id = _native_create_button(text, styles);
            if (onClick) {
                NativeUI.addEventListener("click", (data) => {
                    onClick(data);
                });
            }
            return id;
        },
        Header: (text, styles = {}) => {
            return NativeUI.createNode("Text", {
                padding: "20px",
                ...styles
            }, text);
        },
        Card: (children, styles = {}) => {
            return NativeUI.createNode("Box", {
                padding: "15px",
                margin: "10px",
                ...styles
            });
        }
    }
};

globalThis._native_on_event = (type, data) => {
    if (_eventListeners[type]) {
        _eventListeners[type].forEach(callback => callback(data));
    }
};

console.log("QuickJS: NativeUI bridge initialized.");

// Autonomous Task Scheduler
const SCHEDULER_INTERVAL = 60000; // 60 seconds

function runAutonomousMaintenance() {
    console.log("Scheduler: Running autonomous maintenance...");
    NativeUI.healthCheck();

    const meta = NativeUI.getMetadata();
    const stats = NativeUI.getPerformanceStats();
    console.log(`Scheduler: Active on version ${meta.version} | Stats: FPS=${stats.fps}`);

    if (stats.fps < 10) {
        console.warn("Scheduler: Performance drop detected. Capturing diagnostic screenshot...");
        NativeUI.screenshot("perf_diag.png");
    }

    // Dynamic Auto-Scaling Trigger
    if (stats.fps > 55) {
        NativeUI.scaleResources(500, 1000, 200);
    } else if (stats.fps < 30) {
        NativeUI.scaleResources(50, 100, 20);
    }
}

setInterval(runAutonomousMaintenance, SCHEDULER_INTERVAL);

// Stress Churn Simulation
let churnCount = 0;
function simulateAppChurn() {
    // Dynamically create and immediately "reload" (clear) nodes to simulate activity
    for (let i = 0; i < 50; i++) {
        NativeUI.createNode("Box", { padding: "2px", width: "10px", height: "10px" });
    }
    churnCount++;
    if (churnCount % 10 === 0) {
        console.log(`Churn: Simulated ${churnCount * 50} node operations.`);
        NativeUI.reload(); // Trigger cache test
    }
}

if (globalThis.PROD_MODE) {
    setInterval(simulateAppChurn, 2000);
}

// Performance test logic
const NODE_COUNTS = [100, 500, 1000];

NODE_COUNTS.forEach(count => {
    const start = Date.now();
    for (let i = 0; i < count; i++) {
        NativeUI.createNode("Box", { padding: "1px" });
    }
    const end = Date.now();
    console.log(`JS Performance: Created ${count} nodes in ${end - start}ms`);
});
