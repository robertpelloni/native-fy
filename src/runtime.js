const _eventListeners = {};
const _timers = [];

globalThis.setInterval = (cb, ms) => {
    _timers.push({ cb, ms, last: Date.now() });
};

globalThis.setTimeout = (cb, ms) => {
    _timers.push({ cb, ms, last: Date.now(), once: true });
};

globalThis._native_tick = () => {
    const now = Date.now();
    for (let i = _timers.length - 1; i >= 0; i--) {
        const t = _timers[i];
        if (now - t.last >= t.ms) {
            try {
                t.cb();
            } catch (e) {
                console.error("Timer Error:", e);
            }
            if (t.once) {
                _timers.splice(i, 1);
            } else {
                t.last = now;
            }
        }
    }
};

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
    nativefy: (url) => {
        _native_nativefy(url);
    },
    getMetadata: () => {
        return _native_get_metadata();
    },
    getPerformanceStats: () => {
        return _native_get_perf_stats();
    },
    getSystemMetrics: () => {
        return _native_get_system_metrics();
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
    createSvg: (content, styles) => {
        return _native_create_svg(content, styles);
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
        Svg: (content, styles = {}) => {
            return NativeUI.createSvg(content, styles);
        },
        Input: (placeholder, styles = {}) => {
            return _native_create_input(placeholder, styles);
        },
        List: (itemCount, styles = {}) => {
            return _native_create_list(itemCount, styles);
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
const SCHEDULER_INTERVAL = globalThis.VALIDATION_MODE ? 5000 : 60000;
let _maintenanceIteration = 0;

function runAutonomousMaintenance() {
    _maintenanceIteration++;
    console.log(`Scheduler: Running autonomous maintenance pass (Iter: ${_maintenanceIteration})`);

    // Basic health heartbeat and diagnostic checks
    // Resource orchestration is now handled by the Native Monitor (src/monitor.rs)
    NativeUI.healthCheck();

    const stats = NativeUI.getPerformanceStats();
    if (stats.fps < 10) {
        console.warn("Scheduler: Critical performance drop. Capturing diagnostic screenshot...");
        NativeUI.screenshot("perf_diag.png");
    }

    // Trigger Protocol Sync every 5 minutes (5 iterations)
    if (_maintenanceIteration % 5 === 0) {
        console.log("Scheduler: Triggering scheduled protocol synchronization.");
        NativeUI.syncProtocol();
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
