# CHANGELOG

## [0.38.0] - $(date +%Y-%m-%d)
- Automated synchronization from upstream.
- Conducted intelligent merge resolution on active feature branches.

$(cat CHANGELOG.md | grep -v "# CHANGELOG")
- Implemented Python language bindings using `pyo3` allowing developers to execute Python logic scripts that interface with the `NativeUI` Rust bridge.
- Refined the Event Bridging layer by implementing precise tree-based `hit_test` logic to map cursor and click events strictly to target nodes rather than generic window coordinates.
