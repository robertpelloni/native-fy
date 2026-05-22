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
    }
};

// Internal global called from Rust
globalThis._native_on_event = (type, data) => {
    if (_eventListeners[type]) {
        _eventListeners[type].forEach(callback => callback(data));
    }
};

console.log("QuickJS: NativeUI bridge initialized.");
