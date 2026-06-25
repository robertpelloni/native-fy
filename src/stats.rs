use std::fs::OpenOptions;
use std::io::Write;
use chrono;

pub const LOG_FILE: &str = "app.log";

#[derive(serde::Serialize, Clone, Copy, Default)]
pub struct AppStats {
    pub fps: u32,
    pub layout_time_micros: u64,
    pub node_count: usize,
    pub frame_time_micros: u64,
    pub bridge_time_micros: u64,
    pub render_time_micros: u64,
    pub gpu_time_micros: u64,
    pub hit_test_time_micros: u64,
    pub process_memory_rss_bytes: u64,
    pub cpu_usage: f64,
    pub total_memory: u64,
    pub scheduler_iteration: u32,
    pub batch_size: u32,
    pub text_cache_size: usize,
    pub texture_cache_size: usize,
}

pub fn log_error(msg: &str) {
    let file_res = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE);

    if let Ok(mut file) = file_res {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(file, "[{}] ERROR: {}", timestamp, msg);
    }
    eprintln!("{}", msg);
}
