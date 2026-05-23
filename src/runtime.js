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
    healthCheck: () => {
        _native_health_check();
    },
    reload: () => {
        _native_reload();
    },
    runPipeline: () => {
        _native_run_pipeline();
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
    console.log(`Scheduler: Active on version ${meta.version}`);

    // If we were in a failed state, we would trigger NativeUI.reload() or NativeUI.syncProtocol()
}

setInterval(runAutonomousMaintenance, SCHEDULER_INTERVAL);

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
