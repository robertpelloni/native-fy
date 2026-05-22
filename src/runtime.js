const NativeUI = {
    createNode: (type, styles, text) => {
        // Native bridge call to Rust
        return _native_create_node(type, styles, text);
    },
    setStyle: (nodeId, styles) => {
        // Native bridge call to Rust
        return _native_set_style(nodeId, styles);
    }
};

// Example usage
// const root = NativeUI.createNode("Box", { flexDirection: "column" });
// NativeUI.createNode("Text", { padding: "10px" }, "Hello from QuickJS!");
