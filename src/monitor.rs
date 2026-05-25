use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use sysinfo::System;
use crate::runtime::UiCommand;
use std::sync::mpsc::Sender;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::stats::AppStats;

pub struct Monitor {
    tx: Sender<UiCommand>,
    fps_val: Arc<AtomicU32>,
    sys: Arc<Mutex<System>>,
    current_stats: Arc<Mutex<AppStats>>,
}

impl Monitor {
    pub fn new(tx: Sender<UiCommand>, fps_val: Arc<AtomicU32>, sys: Arc<Mutex<System>>, current_stats: Arc<Mutex<AppStats>>) -> Self {
        Self { tx, fps_val, sys, current_stats }
    }

    pub fn spawn(self) {
        std::thread::spawn(move || {
            let mut last_decision = Instant::now();
            loop {
                std::thread::sleep(Duration::from_millis(500));

                let stats = {
                    let shared = self.current_stats.lock().unwrap();
                    *shared
                };

                let fps = self.fps_val.load(Ordering::Relaxed);
                let (cpu_usage, _total_mem, _used_mem) = {
                    let mut sys = self.sys.lock().unwrap();
                    sys.refresh_cpu_usage();
                    sys.refresh_memory();
                    (sys.global_cpu_usage(), sys.total_memory(), sys.used_memory())
                };

                if last_decision.elapsed() > Duration::from_secs(2) {
                    let mut batch_size = 100;
                    let mut text_threshold = 200;
                    let mut texture_threshold = 50;

                    if fps > 55 && cpu_usage < 70.0 && stats.layout_time_micros < 1000 {
                        // High performance headroom: Aggressive scaling
                        batch_size = 500;
                        text_threshold = 1000;
                        texture_threshold = 200;
                    } else if fps < 30 || cpu_usage > 90.0 || stats.layout_time_micros > 5000 {
                        // Pressure detected: Tighten resources
                        batch_size = 50;
                        text_threshold = 100;
                        texture_threshold = 20;
                    }

                    let _ = self.tx.send(UiCommand::ScaleResources {
                        batch_size,
                        text_eviction_threshold: text_threshold,
                        texture_eviction_threshold: texture_threshold,
                    });

                    last_decision = Instant::now();
                }
            }
        });
    }
}
