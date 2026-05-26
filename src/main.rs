mod layout;
mod ui_gen;
mod runtime;
mod monitor;
mod stats;
mod render;
mod app;

use std::time::{Instant};
use stats::{log_error, AppStats};
use app::NativefyApp;
use crate::layout::LayoutEngine;
use winit::event_loop::EventLoop;

fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };
        let location = panic_info.location().map(|l| format!(" at {}:{}", l.file(), l.line())).unwrap_or_default();
        log_error(&format!("PANIC: {}{}", msg, location));
    }));

    println!("Initializing Native-fy Windowing Environment...");

    if std::env::var("BENCHMARK_MODE").is_ok() {
        let mut engine = LayoutEngine::new();
        let start = Instant::now();
        let root_id = ui_gen::generate_ui_tree(&mut engine);
        let _ = engine.compute(root_id);
        let layout_time = start.elapsed().as_micros() as u64;

        let stats = AppStats {
            fps: 0,
            layout_time_micros: layout_time,
            node_count: engine.node_count(),
            frame_time_micros: 0,
            bridge_time_micros: 0,
            render_time_micros: 0,
            gpu_time_micros: 0,
            process_memory_rss_bytes: 0,
            cpu_usage: 0.0,
            total_memory: 0,
            scheduler_iteration: 0,
        };
        let json = serde_json::to_string_pretty(&stats).unwrap();
        let _ = std::fs::write("perf_metrics.json", json);
        println!("Headless Benchmark: Calculated real layout timings.");
        return;
    }

    // Initialize winit event loop
    let event_loop_res = EventLoop::new();
    let event_loop = match event_loop_res {
        Ok(el) => el,
        Err(e) => {
            log_error(&format!("Failed to create event loop: {}", e));
            return;
        }
    };

    let mut app = NativefyApp::default();

    println!("Starting event loop...");
    let _ = event_loop.run_app(&mut app);
}
