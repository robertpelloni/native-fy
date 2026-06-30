use rquickjs::{Context, Runtime, Function};
use std::sync::mpsc::Sender;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};

pub enum UiCommand {
    CreateNode {
        node_type: String,
        styles: HashMap<String, String>,
        text: Option<String>,
    },
    CreateNativeButton {
        text: String,
        styles: HashMap<String, String>,
    },
    CreateNativeInput {
        placeholder: String,
        styles: HashMap<String, String>,
    },
    CreateNativeList {
        item_count: u32,
        styles: HashMap<String, String>,
    },
    UpdateImage {
        url: String,
        data: Vec<u8>,
    },
    SyncProtocol,
    Nativefy { url: String },
    HealthCheck,
    Reload,
    RunPipeline,
    RunAutonomousTask,
    Svg { content: String, styles: HashMap<String, String> },
    Screenshot { path: String },
    ToggleDashboard,
    ScaleResources {
        batch_size: u32,
        text_eviction_threshold: usize,
        texture_eviction_threshold: usize,
    },
    HotReloadScript { script: String },
    PlayAudio { id: String, url: String },
    StopAudio { id: String },
}

pub struct JsRuntime {
    pub runtime: Runtime,
    pub context: Context,
}

impl JsRuntime {
    pub fn new(tx: Sender<UiCommand>, fps_val: Arc<AtomicU32>, sys: Arc<Mutex<sysinfo::System>>) -> Self {
        let runtime = Runtime::new().expect("failed to create QuickJS runtime");
        let context = Context::full(&runtime).expect("failed to create QuickJS context");

        // Set up the bridge
        context.with(|ctx| {
            let globals = ctx.globals();

            // Native console.log polyfill
            let console = rquickjs::Object::new(ctx.clone()).unwrap();
            console.set("log", Function::new(ctx.clone(), |args: rquickjs::function::Rest<rquickjs::Value>| {
                let mut out = String::new();
                for arg in args.0 {
                    out.push_str(&format!("{:?}", arg));
                    out.push(' ');
                }
                println!("{}", out.trim());
            })).unwrap();
            console.set("warn", Function::new(ctx.clone(), |args: rquickjs::function::Rest<rquickjs::Value>| {
                let mut out = String::new();
                for arg in args.0 {
                    out.push_str(&format!("{:?}", arg));
                    out.push(' ');
                }
                eprintln!("WARN: {}", out.trim());
            })).unwrap();
            console.set("error", Function::new(ctx.clone(), |args: rquickjs::function::Rest<rquickjs::Value>| {
                let mut out = String::new();
                for arg in args.0 {
                    out.push_str(&format!("{:?}", arg));
                    out.push(' ');
                }
                eprintln!("ERROR: {}", out.trim());
            })).unwrap();
            globals.set("console", console).unwrap();

            let tx_create = tx.clone();
            globals.set("_native_create_node", Function::new(ctx.clone(), move |_type: String, _styles: rquickjs::Object, _text: Option<String>| {
                let mut styles = HashMap::new();

                // Extract styles from JS object
                for key_res in _styles.keys::<String>() {
                    if let Ok(key) = key_res
                        && let Ok(val) = _styles.get::<String, String>(key.clone()) {
                            styles.insert(key, val);
                        }
                }

                let _ = tx_create.send(UiCommand::CreateNode {
                    node_type: _type,
                    styles,
                    text: _text,
                });
                0 // Placeholder NodeId
            })).unwrap();

            globals.set("_native_set_style", Function::new(ctx.clone(), |_node_id: u32, _styles: rquickjs::Object| {
                // println!("Native: Setting style for node {}", _node_id);
            })).unwrap();

            let tx_fetch = tx.clone();
            globals.set("_native_fetch", Function::new(ctx.clone(), move |url: String| {
                let tx = tx_fetch.clone();
                let url_clone = url.clone();
                std::thread::spawn(move || {
                    if let Ok(resp) = reqwest::blocking::get(&url_clone)
                        && let Ok(bytes) = resp.bytes() {
                            let _ = tx.send(UiCommand::UpdateImage {
                                url: url_clone,
                                data: bytes.to_vec(),
                            });
                        }
                });
                "Asset loading started...".to_string()
            })).unwrap();

            let tx_sync = tx.clone();
            globals.set("_native_sync_protocol", Function::new(ctx.clone(), move || {
                let _ = tx_sync.send(UiCommand::SyncProtocol);
            })).unwrap();

            let tx_nfy = tx.clone();
            globals.set("_native_nativefy", Function::new(ctx.clone(), move |url: String| {
                let _ = tx_nfy.send(UiCommand::Nativefy { url });
            })).unwrap();

            let tx_play = tx.clone();
            globals.set("_native_play_audio", Function::new(ctx.clone(), move |id: String, url: String| {
                let _ = tx_play.send(UiCommand::PlayAudio { id, url });
            })).unwrap();

            let tx_stop = tx.clone();
            globals.set("_native_stop_audio", Function::new(ctx.clone(), move |id: String| {
                let _ = tx_stop.send(UiCommand::StopAudio { id });
            })).unwrap();

            let tx_btn = tx.clone();
            globals.set("_native_create_button", Function::new(ctx.clone(), move |text: String, _styles: rquickjs::Object| {
                let mut styles = HashMap::new();
                for key_res in _styles.keys::<String>() {
                    if let Ok(key) = key_res
                        && let Ok(val) = _styles.get::<String, String>(key.clone()) {
                            styles.insert(key, val);
                        }
                }
                let _ = tx_btn.send(UiCommand::CreateNativeButton { text, styles });
            })).unwrap();

            let tx_input = tx.clone();
            globals.set("_native_create_input", Function::new(ctx.clone(), move |placeholder: String, _styles: rquickjs::Object| {
                let mut styles = HashMap::new();
                for key_res in _styles.keys::<String>() {
                    if let Ok(key) = key_res
                        && let Ok(val) = _styles.get::<String, String>(key.clone()) {
                            styles.insert(key, val);
                        }
                }
                let _ = tx_input.send(UiCommand::CreateNativeInput { placeholder, styles });
            })).unwrap();

            let tx_list = tx.clone();
            globals.set("_native_create_list", Function::new(ctx.clone(), move |item_count: u32, _styles: rquickjs::Object| {
                let mut styles = HashMap::new();
                for key_res in _styles.keys::<String>() {
                    if let Ok(key) = key_res
                        && let Ok(val) = _styles.get::<String, String>(key.clone()) {
                            styles.insert(key, val);
                        }
                }
                let _ = tx_list.send(UiCommand::CreateNativeList { item_count, styles });
            })).unwrap();

            let tx_health = tx.clone();
            globals.set("_native_health_check", Function::new(ctx.clone(), move || {
                let _ = tx_health.send(UiCommand::HealthCheck);
            })).unwrap();

            let tx_reload = tx.clone();
            globals.set("_native_reload", Function::new(ctx.clone(), move || {
                let _ = tx_reload.send(UiCommand::Reload);
            })).unwrap();

            let tx_svg = tx.clone();
            globals.set("_native_create_svg", Function::new(ctx.clone(), move |content: String, _styles: rquickjs::Object| {
                let mut styles = HashMap::new();
                for key_res in _styles.keys::<String>() {
                    if let Ok(key) = key_res
                        && let Ok(val) = _styles.get::<String, String>(key.clone()) {
                            styles.insert(key, val);
                        }
                }
                let _ = tx_svg.send(UiCommand::Svg { content, styles });
            })).unwrap();

            let tx_pipe = tx.clone();
            globals.set("_native_run_pipeline", Function::new(ctx.clone(), move || {
                let _ = tx_pipe.send(UiCommand::RunPipeline);
            })).unwrap();

            let tx_task = tx.clone();
            globals.set("_native_run_autonomous_task", Function::new(ctx.clone(), move || {
                let _ = tx_task.send(UiCommand::RunAutonomousTask);
            })).unwrap();

            let tx_ss = tx.clone();
            globals.set("_native_screenshot", Function::new(ctx.clone(), move |path: String| {
                let _ = tx_ss.send(UiCommand::Screenshot { path });
            })).unwrap();

            let tx_dash = tx.clone();
            globals.set("_native_toggle_dashboard", Function::new(ctx.clone(), move || {
                let _ = tx_dash.send(UiCommand::ToggleDashboard);
            })).unwrap();

            let tx_scale = tx.clone();
            globals.set("_native_scale_resources", Function::new(ctx.clone(), move |batch_size: u32, text_threshold: usize, texture_threshold: usize| {
                let _ = tx_scale.send(UiCommand::ScaleResources {
                    batch_size,
                    text_eviction_threshold: text_threshold,
                    texture_eviction_threshold: texture_threshold,
                });
            })).unwrap();

            globals.set("_native_get_metadata", Function::new(ctx.clone(), || {
                let version = include_str!("../VERSION.md").trim();
                let todo = include_str!("../TODO.md");
                let mut meta = HashMap::new();
                meta.insert("version".to_string(), version.to_string());
                meta.insert("todo".to_string(), todo.to_string());
                meta
            })).unwrap();

            let fps_clone = fps_val.clone();
            globals.set("_native_get_perf_stats", Function::new(ctx.clone(), move || {
                let mut stats = HashMap::new();
                stats.insert("fps".to_string(), fps_clone.load(Ordering::Relaxed) as f64);
                stats.insert("latency".to_string(), 0.0); // Placeholder for latency
                stats
            })).unwrap();

            let sys_clone = sys.clone();
            globals.set("_native_get_system_metrics", Function::new(ctx.clone(), move || {
                let mut sys = sys_clone.lock().unwrap();
                sys.refresh_cpu_usage();
                sys.refresh_memory();

                let mut metrics = HashMap::new();
                metrics.insert("cpu_usage".to_string(), sys.global_cpu_usage() as f64);
                metrics.insert("total_mem".to_string(), sys.total_memory() as f64);
                metrics.insert("used_mem".to_string(), sys.used_memory() as f64);
                metrics
            })).unwrap();
        });

        Self {
            runtime,
            context,
        }
    }

    pub fn eval(&self, source: &str) {
        self.context.with(|ctx| {
            let res: Result<rquickjs::Value, _> = ctx.eval(source);
            if let Err(e) = res {
                if let Some(exception) = ctx.catch().as_exception() {
                    eprintln!("JS Exception: {} at {}",
                        exception.message().unwrap_or_default(),
                        exception.stack().unwrap_or_default()
                    );
                } else {
                    eprintln!("JS Error: {:?}", e);
                }
            }
        });
    }

    pub fn tick(&self) {
        self.context.with(|ctx| {
            let globals = ctx.globals();
            if let Ok(handler) = globals.get::<_, Function>("_native_tick") {
                let _ = handler.call::<(), ()>(());
            }
        });
    }

    pub fn dispatch_click(&self, x: f32, y: f32, target_node_id: Option<u64>) {
        self.context.with(|ctx| {
            let globals = ctx.globals();
            if let Ok(handler) = globals.get::<_, Function>("_native_on_event") {
                let data = rquickjs::Object::new(ctx.clone()).unwrap();
                let _ = data.set("x", x);
                let _ = data.set("y", y);
                if let Some(id) = target_node_id {
                    let _ = data.set("targetId", id as f64);
                }
                let _ = handler.call::<(String, rquickjs::Object), ()>(("click".to_string(), data));
            }
        });
    }

    pub fn dispatch_cursor(&self, x: f32, y: f32, target_node_id: Option<u64>) {
        self.context.with(|ctx| {
            let globals = ctx.globals();
            if let Ok(handler) = globals.get::<_, Function>("_native_on_event") {
                let data = rquickjs::Object::new(ctx.clone()).unwrap();
                let _ = data.set("x", x);
                let _ = data.set("y", y);
                if let Some(id) = target_node_id {
                    let _ = data.set("targetId", id as f64);
                }
                let _ = handler.call::<(String, rquickjs::Object), ()>(("mousemove".to_string(), data));
            }
        });
    }

    pub fn update_stats(&self, stats: &crate::stats::AppStats) {
        self.context.with(|ctx| {
            let globals = ctx.globals();
            let data = rquickjs::Object::new(ctx.clone()).unwrap();
            let _ = data.set("fps", stats.fps);
            let _ = data.set("cpu_usage", stats.cpu_usage);
            let _ = data.set("memory_usage_percent", (stats.process_memory_rss_bytes as f64 / stats.total_memory as f64) * 100.0);
            let _ = data.set("batch_size", stats.batch_size);
            let _ = data.set("layout_time_micros", stats.layout_time_micros);

            // Push these explicitly to globalThis._latest_stats in QuickJS
            let _ = globals.set("_latest_stats", data);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Instant;

    #[test]
    fn test_js_node_creation_performance() {
        let (tx, rx) = mpsc::channel();
        let fps = Arc::new(AtomicU32::new(60));
        let sys = Arc::new(Mutex::new(sysinfo::System::new_all()));
        let runtime = JsRuntime::new(tx, fps, sys);

        let counts = [100, 500, 1000];
        for count in counts {
            let start = Instant::now();
            runtime.eval(&format!(
                "for (let i = 0; i < {}; i++) {{ _native_create_node('Box', {{ padding: '10px' }}, null); }}",
                count
            ));
            let duration = start.elapsed();
            println!("JS Performance: Created {} nodes with styles in {:?}", count, duration);

            // Verify nodes and styles received in channel
            for _ in 0..count {
                if let UiCommand::CreateNode { styles, .. } = rx.try_recv().expect("node not received") {
                    assert_eq!(styles.get("padding").unwrap(), "10px");
                }
            }
        }
    }
}
