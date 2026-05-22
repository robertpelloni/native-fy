# PERFORMANCE

## Benchmarking Results

The following benchmarks measure the efficiency of the QuickJS-to-Native bridge and the core rendering pipeline.

### Node Creation (JS Bridge)
Measured using `cargo test -- --nocapture` on the `JsRuntime` node creation loop.

| Node Count | Duration |
|------------|----------|
| 100        | 366µs    |
| 500        | 1.39ms   |
| 1000       | 2.45ms   |

*Analysis:* The bridge overhead is extremely low, allowing for the dynamic generation of complex UI trees within a single frame's budget (16.6ms for 60fps).

### Rendering & Layout
Initial instrumentation results from `src/main.rs`.

- **Initial Layout Computation:** ~1.2ms (Simple tree)
- **Per-Frame Render Pass:** ~0.8ms (10 nodes, box + text)

## Target Metrics
- **Max Nodes:** Support 10,000 nodes at > 60fps.
- **Input Latency:** < 10ms from native event to JS handler.
