use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use notify::Watcher;

use std::sync::mpsc::{Receiver, Sender};
use std::sync::atomic::{Ordering};

use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, ElementState, MouseButton},
    event_loop::{ActiveEventLoop},
    window::{Window, WindowId},
};

use crate::layout::{LayoutEngine, Node, AstRect, FlexStyles};
use crate::runtime::{JsRuntime, UiCommand};
use crate::render::{RenderState, NodeData};
use crate::stats::{AppStats, log_error};
use crate::{ui_gen, monitor};

const VERIFICATION_UI_SCRIPT: &str = r#"
    NativeUI.Components.Button("Trigger Reload", () => {
        console.log("UI: Reload triggered from button");
        NativeUI.reload();
    }, { margin: "10px" });

    NativeUI.Components.Button("Test Fetch", async () => {
        console.log("UI: Fetch triggered from button");
        const data = await NativeUI.fetch("https://google.com");
        console.log("UI: Fetch result received");
    }, { margin: "10px" });

    NativeUI.Components.Button("Capture Frame", () => {
        console.log("UI: Screenshot triggered from button");
        NativeUI.screenshot("manual_capture.png");
    }, { margin: "10px" });

    NativeUI.Components.Svg('<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg"><circle cx="50" cy="50" r="40" stroke="green" stroke-width="4" fill="yellow" /></svg>', { width: "100px", height: "100px", margin: "10px" });
"#;


pub struct NativefyApp {
    pub window: Option<Arc<Window>>,
    pub fps_val: Arc<std::sync::atomic::AtomicU32>,
    pub sys: Arc<Mutex<sysinfo::System>>,
    pub current_stats: Arc<Mutex<AppStats>>,
    pub render_state: Option<RenderState>,
    pub layout_engine: Option<LayoutEngine>,
    pub root_id: Option<taffy::prelude::NodeId>,
    pub js_runtime: Option<JsRuntime>,
    pub mouse_pos: [f32; 2],
    pub ui_rx: Receiver<UiCommand>,
    pub ui_tx: Sender<UiCommand>,
    pub pending_screenshot: Option<String>,
    pub last_frame: Instant,
    pub fps: u32,
    pub frame_count: u32,
    pub last_fps_update: Instant,
    pub perf_history: Vec<AppStats>,
    pub dashboard_active: bool,
    pub batch_size: u32,
}

impl Default for NativefyApp {
    fn default() -> Self {
        let (ui_tx, ui_rx) = std::sync::mpsc::channel();
        Self {
            window: None,
            fps_val: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            sys: Arc::new(Mutex::new(sysinfo::System::new_all())),
            current_stats: Arc::new(Mutex::new(AppStats::default())),
            render_state: None,
            layout_engine: None,
            root_id: None,
            js_runtime: None,
            mouse_pos: [0.0; 2],
            ui_rx,
            ui_tx,
            pending_screenshot: None,
            last_frame: Instant::now(),
            fps: 0,
            frame_count: 0,
            last_fps_update: Instant::now(),
            perf_history: Vec::new(),
            dashboard_active: std::env::var("DASHBOARD_MODE").is_ok(),
            batch_size: 100,
        }
    }
}

impl ApplicationHandler for NativefyApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Native-fy UI Engine")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));

            let window_res = event_loop.create_window(window_attributes);
            if let Ok(window) = window_res {
                let window = Arc::new(window);
                self.window = Some(window.clone());

                let render_state_res = pollster::block_on(RenderState::new(window));
                match render_state_res {
                    Ok(state) => self.render_state = Some(state),
                    Err(e) => {
                        log_error(&format!("Failed to initialize render state: {}", e));
                    }
                }
            } else {
                log_error("Failed to create window (Headless environment detected).");
            }

            let mut engine = LayoutEngine::new();

            let layout_start = Instant::now();
            let root_id = ui_gen::generate_ui_tree(&mut engine);
            let _ = engine.compute(root_id);
            let _layout_duration = layout_start.elapsed();
            #[cfg(debug_assertions)]
            if !std::env::var("PROD_MODE").is_ok() {
                println!("Performance: Initial layout computed in {:?}", _layout_duration);
            }

            self.layout_engine = Some(engine);
            self.root_id = Some(root_id);

            // Initialize Monitor
            let monitor = monitor::Monitor::new(
                self.ui_tx.clone(),
                self.fps_val.clone(),
                self.sys.clone(),
                self.current_stats.clone()
            );
            monitor.spawn();

            // Initialize QuickJS
            let runtime = JsRuntime::new(self.ui_tx.clone(), self.fps_val.clone(), self.sys.clone());
            let mut bridge_code = include_str!("runtime.js").to_string();
            if std::env::var("PROD_MODE").is_ok() {
                bridge_code = format!("globalThis.PROD_MODE = true; \n {}", bridge_code);
            }
            if std::env::var("VALIDATION_MODE").is_ok() {
                bridge_code = format!("globalThis.VALIDATION_MODE = true; \n {}", bridge_code);
            }
            runtime.eval(&bridge_code);



            // Hot-reloading watcher
            let watch_tx = self.ui_tx.clone();
            std::thread::spawn(move || {
                let (tx, rx) = std::sync::mpsc::channel();
                let mut watcher = notify::recommended_watcher(tx).unwrap();
                let mut js_path = std::env::current_dir().unwrap().join("src/runtime.js");

                if !js_path.exists() {
                     // Fallback for execution from target dir (like in tests)
                     let alt_path = std::env::current_exe()
                        .unwrap_or_else(|_| std::env::current_dir().unwrap())
                        .parent().unwrap()
                        .parent().unwrap()
                        .parent().unwrap()
                        .join("src/runtime.js");
                     if alt_path.exists() {
                         js_path = alt_path;
                     }
                }

                if js_path.exists() {
                    watcher.watch(&js_path, notify::RecursiveMode::NonRecursive).unwrap();

                    for res in rx {
                        match res {
                            Ok(event) => {
                                if let notify::EventKind::Modify(_) = event.kind {
                                    if let Ok(script) = std::fs::read_to_string(&js_path) {
                                        let _ = watch_tx.send(UiCommand::HotReloadScript { script });
                                    }
                                }
                            },
                            Err(e) => crate::stats::log_error(&format!("Watch error: {:?}", e)),
                        }
                    }
                } else {
                    println!("Hot reload watcher failed to find src/runtime.js at {:?}", js_path);
                }
            });

            // Wire bridge features to UI for verification


            runtime.eval(VERIFICATION_UI_SCRIPT);

            self.js_runtime = Some(runtime);

            if !std::env::var("PROD_MODE").is_ok() {
                println!("Window, Wgpu, and QuickJS successfully initialized!");
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(state) = self.render_state.as_mut() {
                    state.resize(physical_size);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_pos = [position.x as f32, position.y as f32];
                let hit_test_start = Instant::now();
                if let (Some(engine), Some(root_id)) = (self.layout_engine.as_ref(), self.root_id) {
                    let hit_id = engine.hit_test(root_id, self.mouse_pos[0], self.mouse_pos[1]);
                    if let Some(runtime) = self.js_runtime.as_ref() {
                        let target_id = hit_id.map(|id| u64::from(id));
                        let bridge_start = Instant::now();
                        runtime.dispatch_cursor(self.mouse_pos[0], self.mouse_pos[1], target_id);
                        if let Ok(mut stats) = self.current_stats.lock() {
                            stats.bridge_time_micros = stats.bridge_time_micros.max(bridge_start.elapsed().as_micros() as u64);
                        }
                    }
                }
                if let Ok(mut stats) = self.current_stats.lock() {
                    stats.hit_test_time_micros = stats.hit_test_time_micros.max(hit_test_start.elapsed().as_micros() as u64);
                }
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                let hit_test_start = Instant::now();
                if let (Some(engine), Some(root_id)) = (self.layout_engine.as_ref(), self.root_id) {
                    let hit_id = engine.hit_test(root_id, self.mouse_pos[0], self.mouse_pos[1]);
                    if let Some(runtime) = self.js_runtime.as_ref() {
                        let target_id = hit_id.map(|id| u64::from(id));
                        let bridge_start = Instant::now();
                        runtime.dispatch_click(self.mouse_pos[0], self.mouse_pos[1], target_id);
                        if let Ok(mut stats) = self.current_stats.lock() {
                            stats.bridge_time_micros = stats.bridge_time_micros.max(bridge_start.elapsed().as_micros() as u64);
                        }
                    }
                }
                if let Ok(mut stats) = self.current_stats.lock() {
                    stats.hit_test_time_micros = stats.hit_test_time_micros.max(hit_test_start.elapsed().as_micros() as u64);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(runtime) = self.js_runtime.as_ref() {
                    runtime.tick();
                }
                let _loop_start = Instant::now();
                // Update FPS
                self.frame_count += 1;
                let now = Instant::now();
                if now.duration_since(self.last_fps_update) >= Duration::from_secs(1) {
                    self.fps = self.frame_count;
                    self.fps_val.store(self.fps, Ordering::Relaxed);
                    self.frame_count = 0;
                    self.last_fps_update = now;
                }

                let bridge_start = Instant::now();
                // Process UI commands in batch
                let mut recompute = false;
                let mut command_count = 0;
                while let Ok(cmd) = self.ui_rx.try_recv() {
                    command_count += 1;
                    if command_count > self.batch_size { break; } // Safety break
                    match cmd {
                        UiCommand::CreateNode { node_type, styles, text } => {
                            if let (Some(engine), Some(root_id)) = (self.layout_engine.as_mut(), self.root_id) {
                                let width = styles.get("width").and_then(|v| v.strip_suffix("px")).and_then(|v| v.parse().ok()).unwrap_or(100.0);
                                let height = styles.get("height").and_then(|v| v.strip_suffix("px")).and_then(|v| v.parse().ok()).unwrap_or(100.0);
                                let new_node = Node {
                                    node_type,
                                    rect: AstRect { x: 0.0, y: 0.0, width, height },
                                    styles: FlexStyles {
                                        flex_direction: styles.get("flexDirection").cloned().unwrap_or("row".to_string()),
                                        padding: styles.get("padding").cloned().unwrap_or("0px".to_string()),
                                        margin: styles.get("margin").cloned().unwrap_or("0px".to_string()),
                                        align_items: styles.get("alignItems").cloned().unwrap_or("stretch".to_string()),
                                        justify_content: styles.get("justifyContent").cloned().unwrap_or("flex-start".to_string()),
                                        unsupported: std::collections::HashMap::new(),
                                    },
                                    text,
                                    value: None,
                                    children: vec![],
                                };
                                if let Ok(new_id) = engine.build_tree(&new_node) {
                                    let _ = engine.add_child(root_id, new_id);
                                }
                                recompute = true;
                            }
                        }
                        UiCommand::CreateNativeInput { placeholder, styles } => {
                            if let (Some(engine), Some(root_id)) = (self.layout_engine.as_mut(), self.root_id) {
                                let new_node = Node {
                                    node_type: "Input".to_string(),
                                    rect: AstRect { x: 0.0, y: 0.0, width: 200.0, height: 40.0 },
                                    styles: FlexStyles {
                                        flex_direction: "row".to_string(),
                                        padding: styles.get("padding").cloned().unwrap_or("5px 10px".to_string()),
                                        margin: styles.get("margin").cloned().unwrap_or("5px".to_string()),
                                        align_items: "center".to_string(),
                                        justify_content: "flex-start".to_string(),
                                        unsupported: std::collections::HashMap::new(),
                                    },
                                    text: None,
                                    value: Some(placeholder),
                                    children: vec![],
                                };
                                if let Ok(new_id) = engine.build_tree(&new_node) {
                                    let _ = engine.add_child(root_id, new_id);
                                }
                                recompute = true;
                            }
                        }
                        UiCommand::CreateNativeList { item_count, styles } => {
                            if let (Some(engine), Some(root_id)) = (self.layout_engine.as_mut(), self.root_id) {
                                let mut children = Vec::new();
                                for i in 0..item_count {
                                    children.push(Node {
                                        node_type: "Box".to_string(),
                                        rect: AstRect { x: 0.0, y: 0.0, width: 0.0, height: 30.0 },
                                        styles: FlexStyles {
                                            flex_direction: "row".to_string(),
                                            padding: "5px".to_string(),
                                            margin: "2px".to_string(),
                                            align_items: "center".to_string(),
                                            justify_content: "flex-start".to_string(),
                                            unsupported: std::collections::HashMap::new(),
                                        },
                                        text: Some(format!("Item {}", i + 1)),
                                        value: None,
                                        children: vec![],
                                    });
                                }
                                let new_node = Node {
                                    node_type: "List".to_string(),
                                    rect: AstRect { x: 0.0, y: 0.0, width: 200.0, height: 0.0 },
                                    styles: FlexStyles {
                                        flex_direction: "column".to_string(),
                                        padding: styles.get("padding").cloned().unwrap_or("10px".to_string()),
                                        margin: styles.get("margin").cloned().unwrap_or("5px".to_string()),
                                        align_items: "stretch".to_string(),
                                        justify_content: "flex-start".to_string(),
                                        unsupported: std::collections::HashMap::new(),
                                    },
                                    text: None,
                                    value: None,
                                    children,
                                };
                                if let Ok(new_id) = engine.build_tree(&new_node) {
                                    let _ = engine.add_child(root_id, new_id);
                                }
                                recompute = true;
                            }
                        }
                        UiCommand::CreateNativeButton { text, styles } => {
                            #[cfg(debug_assertions)]
                            if !std::env::var("PROD_MODE").is_ok() {
                                println!("Runtime: Creating native button '{}'", text);
                            }
                            if let (Some(engine), Some(root_id)) = (self.layout_engine.as_mut(), self.root_id) {
                                let new_node = Node {
                                    node_type: "Box".to_string(), // Native button is a box with text
                                    rect: AstRect { x: 0.0, y: 0.0, width: 100.0, height: 40.0 }, // Pre-defined button size
                                    styles: FlexStyles {
                                        flex_direction: "row".to_string(),
                                        padding: styles.get("padding").cloned().unwrap_or("10px 20px".to_string()),
                                        margin: styles.get("margin").cloned().unwrap_or("5px".to_string()),
                                        align_items: "center".to_string(),
                                        justify_content: "center".to_string(),
                                        unsupported: std::collections::HashMap::new(),
                                    },
                                    text: Some(text),
                                    value: None,
                                    children: vec![],
                                };
                                if let Ok(new_id) = engine.build_tree(&new_node) {
                                    let _ = engine.add_child(root_id, new_id);
                                }
                                recompute = true;
                            }
                        }
                        UiCommand::UpdateImage { url, data } => {
                            if !std::env::var("PROD_MODE").is_ok() {
                                println!("Runtime: Loading image asset from {}", url);
                            }
                            if let (Some(state), Ok(img)) = (self.render_state.as_mut(), image::load_from_memory(&data)) {
                                let rgba = img.to_rgba8();
                                let (width, height) = rgba.dimensions();

                                let texture_size = wgpu::Extent3d {
                                    width,
                                    height,
                                    depth_or_array_layers: 1,
                                };

                                let texture = state.device.create_texture(&wgpu::TextureDescriptor {
                                    label: Some(&url),
                                    size: texture_size,
                                    mip_level_count: 1,
                                    sample_count: 1,
                                    dimension: wgpu::TextureDimension::D2,
                                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                                    view_formats: &[],
                                });

                                state.queue.write_texture(
                                    wgpu::ImageCopyTexture {
                                        texture: &texture,
                                        mip_level: 0,
                                        origin: wgpu::Origin3d::ZERO,
                                        aspect: wgpu::TextureAspect::All,
                                    },
                                    &rgba,
                                    wgpu::ImageDataLayout {
                                        offset: 0,
                                        bytes_per_row: Some(4 * width),
                                        rows_per_image: Some(height),
                                    },
                                    texture_size,
                                );

                                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                                let sampler = state.device.create_sampler(&wgpu::SamplerDescriptor {
                                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                                    mag_filter: wgpu::FilterMode::Linear,
                                    min_filter: wgpu::FilterMode::Nearest,
                                    mipmap_filter: wgpu::FilterMode::Nearest,
                                    ..Default::default()
                                });

                                let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
                                    label: Some("Texture Bind Group"),
                                    layout: &state.bind_group_layout,
                                    entries: &[
                                        wgpu::BindGroupEntry {
                                            binding: 0,
                                            resource: state.globals_buffer.as_entire_binding(),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 1,
                                            resource: state.node_buffer.as_entire_binding(),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 2,
                                            resource: wgpu::BindingResource::TextureView(&view),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 3,
                                            resource: wgpu::BindingResource::Sampler(&sampler),
                                        },
                                    ],
                                });

                                if state.textures.len() > state.texture_eviction_threshold {
                                    let mut entries: Vec<_> = state.textures.iter().map(|(k, v)| (k.clone(), v.1)).collect();
                                    entries.sort_by_key(|&(_, last_used)| last_used);
                                    let evict_count = (state.texture_eviction_threshold / 5).max(1);
                                    for i in 0..evict_count.min(entries.len()) {
                                        state.textures.remove(&entries[i].0);
                                    }
                                    #[cfg(debug_assertions)]
                                    if !std::env::var("PROD_MODE").is_ok() {
                                        println!("Memory: Evicted {} textures (LRU).", evict_count);
                                    }
                                }
                                state.textures.insert(url, (bind_group, Instant::now()));
                            }
                        }
                        UiCommand::HealthCheck => {
                            println!("Health Check: Bridge is responsive.");
                        }
                        UiCommand::Reload => {
                            if !std::env::var("PROD_MODE").is_ok() {
                                println!("Runtime: Reloading UI tree...");
                            }
                            if let Some(engine) = self.layout_engine.as_mut() {
                                // Memory Safety: Clear existing tree before re-generating
                                engine.clear();
                                let new_root = ui_gen::generate_ui_tree(engine);
                                self.root_id = Some(new_root);
                                let _ = engine.compute(new_root);
                                recompute = true;
                            }
                        }
                        UiCommand::Screenshot { path } => {
                            println!("Runtime: Queuing frame capture to {}", path);
                            self.pending_screenshot = Some(path);
                        }
                        UiCommand::ToggleDashboard => {
                            self.dashboard_active = !self.dashboard_active;
                            println!("Runtime: Dashboard is now {}", if self.dashboard_active { "ACTIVE" } else { "INACTIVE" });
                        }
                        UiCommand::RunPipeline => {
                            println!("Watchdog: Executing Recovery Pipeline...");
                            // In a production environment this would trigger an external watchdog process
                            // or restart the host service. For now, we execute the JS E2E pipeline script.
                            let status = std::process::Command::new("npm")
                                .arg("run")
                                .arg("test:e2e")
                                .status();
                            if let Ok(exit_status) = status {
                                if !exit_status.success() {
                                    log_error("Watchdog: Pipeline Recovery Failed! Initiating hard reboot protocol.");
                                    // Here we would exit the process so the OS supervisor (e.g. systemd/Docker) restarts it.
                                    // std::process::exit(1);
                                } else {
                                    println!("Watchdog: Pipeline Recovery Successful.");
                                }
                            } else {
                                log_error("Watchdog: Failed to execute recovery script.");
                            }
                        }
                        UiCommand::Svg { content, styles } => {
                            if let (Some(engine), Some(root_id)) = (self.layout_engine.as_mut(), self.root_id) {
                                let width = styles.get("width").and_then(|v| v.strip_suffix("px")).and_then(|v| v.parse().ok()).unwrap_or(100.0);
                                let height = styles.get("height").and_then(|v| v.strip_suffix("px")).and_then(|v| v.parse().ok()).unwrap_or(100.0);
                                let new_node = Node {
                                    node_type: "Svg".to_string(),
                                    rect: AstRect { x: 0.0, y: 0.0, width, height },
                                    styles: FlexStyles {
                                        flex_direction: "row".to_string(),
                                        padding: styles.get("padding").cloned().unwrap_or("0px".to_string()),
                                        margin: styles.get("margin").cloned().unwrap_or("0px".to_string()),
                                        align_items: "center".to_string(),
                                        justify_content: "center".to_string(),
                                        unsupported: std::collections::HashMap::new(),
                                    },
                                    text: None,
                                    value: Some(content),
                                    children: vec![],
                                };
                                if let Ok(new_id) = engine.build_tree(&new_node) {
                                    let _ = engine.add_child(root_id, new_id);
                                }
                                recompute = true;
                            }
                        }
                        UiCommand::SyncProtocol => {
                            println!("Runtime: Triggering Protocol Sync...");
                            let _ = std::process::Command::new("node")
                                .arg("scripts/protocol_sync.js")
                                .status();
                        }
                        UiCommand::Nativefy { url } => {
                            println!("Runtime: Triggering Full Pipeline for {}...", url);
                            let status = std::process::Command::new("npm")
                                .arg("run")
                                .arg("pipeline")
                                .arg("--")
                                .arg(url)
                                .status();

                            if let Ok(s) = status {
                                if s.success() {
                                    println!("Runtime: Transpilation successful. Signalling restart for autonomous update.");
                                    event_loop.exit(); // Exit and let wrapper restart
                                }
                            }
                        }
                        UiCommand::ScaleResources { batch_size, text_eviction_threshold, texture_eviction_threshold } => {
                            println!("Runtime: Scaling resources (Batch: {}, Text: {}, Texture: {})", batch_size, text_eviction_threshold, texture_eviction_threshold);
                            self.batch_size = batch_size;
                            if let Some(state) = self.render_state.as_mut() {
                                state.text_eviction_threshold = text_eviction_threshold;
                                state.texture_eviction_threshold = texture_eviction_threshold;
                            }
                        }

                        UiCommand::HotReloadScript { script } => {
                            println!("Runtime: Hot-reloading QuickJS script...");
                            if let Some(_) = self.js_runtime.as_ref() {
                                // Destroy old runtime and create a fresh one to avoid memory leaks/duplicated listeners
                                let runtime = JsRuntime::new(self.ui_tx.clone(), self.fps_val.clone(), self.sys.clone());
                                let mut bridge_code = script;
                                if std::env::var("PROD_MODE").is_ok() {
                                    bridge_code = format!("globalThis.PROD_MODE = true;
 {}", bridge_code);
                                }
                                if std::env::var("VALIDATION_MODE").is_ok() {
                                    bridge_code = format!("globalThis.VALIDATION_MODE = true;
 {}", bridge_code);
                                }
                                runtime.eval(&bridge_code);

                                // Re-wire verification UI features
                                runtime.eval(VERIFICATION_UI_SCRIPT);

                                self.js_runtime = Some(runtime);

                                if let Some(engine) = self.layout_engine.as_mut() {
                                    engine.clear();
                                    let new_root = ui_gen::generate_ui_tree(engine);
                                    self.root_id = Some(new_root);
                                    let _ = engine.compute(new_root);
                                    recompute = true;
                                }
                            }
                        }
                    }

                }

                let bridge_duration = bridge_start.elapsed();

                let mut layout_duration = Duration::from_micros(0);
                if recompute {
                    if let (Some(engine), Some(root_id)) = (self.layout_engine.as_mut(), self.root_id) {
                        let start = Instant::now();
                        let _ = engine.compute(root_id);
                        layout_duration = start.elapsed();
                    }
                }

                if let (Some(state), Some(engine), Some(root_id)) = (self.render_state.as_mut(), self.layout_engine.as_ref(), self.root_id) {
                    let render_start_hot = Instant::now();

                    let mut sys = self.sys.lock().unwrap();
                    sys.refresh_cpu_usage();
                    sys.refresh_memory();

                    let mut stats = AppStats {
                        fps: self.fps,
                        layout_time_micros: layout_duration.as_micros() as u64,
                        node_count: engine.node_count(),
                        frame_time_micros: now.duration_since(self.last_fps_update).as_micros() as u64 / (self.frame_count.max(1) as u64),
                        bridge_time_micros: bridge_duration.as_micros() as u64,
                        render_time_micros: 0,
                        gpu_time_micros: 0,
                        hit_test_time_micros: 0,
                        process_memory_rss_bytes: sys.process(sysinfo::Pid::from_u32(std::process::id())).map(|p: &sysinfo::Process| p.memory()).unwrap_or(0),
                        cpu_usage: sys.global_cpu_usage() as f64,
                        total_memory: sys.total_memory(),
                        scheduler_iteration: 0, // Will be updated if available
                        batch_size: self.batch_size,
                        text_cache_size: state.text_buffers.len(),
                        texture_cache_size: state.textures.len(),
                    };

                    // Record history for dashboard
                    if self.frame_count % 10 == 0 {
                        self.perf_history.push(stats);
                        if self.perf_history.len() > 100 { self.perf_history.remove(0); }
                    }

                    let screenshot_path = self.pending_screenshot.take();
                    let render_duration = render_start_hot.elapsed();
                    stats.render_time_micros = render_duration.as_micros() as u64;

                    {
                        let mut shared_stats = self.current_stats.lock().unwrap();
                        stats.scheduler_iteration = shared_stats.scheduler_iteration;
                        *shared_stats = stats;
                    }

                    if self.dashboard_active {
                        // Render visualization instead of standard UI
                        let mut dashboard_nodes = Vec::new();
                        let max_fps = 120.0;
                        let max_layout = 5000.0; // 5ms

                        for (i, entry) in self.perf_history.iter().enumerate() {
                            let x = 10.0 + (i as f32 * 7.0);

                            // FPS Bar
                            let fps_h = (entry.fps as f32 / max_fps) * 100.0;
                            dashboard_nodes.push(NodeData {
                                pos: [x, 200.0 - fps_h],
                                size: [5.0, fps_h],
                                color: [0.0, 1.0, 0.0, 1.0],
                                mode: 0,
                                _padding: [0.0; 3],
                            });

                            // Layout Latency Bar
                            let lat_h = (entry.layout_time_micros as f32 / max_layout) * 100.0;
                            dashboard_nodes.push(NodeData {
                                pos: [x, 400.0 - lat_h],
                                size: [5.0, lat_h],
                                color: [1.0, 0.5, 0.0, 1.0],
                                mode: 0,
                                _padding: [0.0; 3],
                            });

                            // Bridge Latency Bar
                            let bridge_h = (entry.bridge_time_micros as f32 / 2000.0) * 50.0;
                            dashboard_nodes.push(NodeData {
                                pos: [x, 500.0 - bridge_h],
                                size: [5.0, bridge_h],
                                color: [0.0, 0.5, 1.0, 1.0],
                                mode: 0,
                                _padding: [0.0; 3],
                            });
                        }

                        state.queue.write_buffer(&state.node_buffer, 0, bytemuck::cast_slice(&dashboard_nodes));
                        let _ = state.render_dashboard(&stats, dashboard_nodes.len() as u32, screenshot_path);
                    } else {
                        match state.render(engine, root_id, &stats, screenshot_path) {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => state.resize(state.size),
                            Err(e) => log_error(&format!("Render error: {:?}", e)),
                        }
                    }

                    // Benchmark export trigger
                    if std::env::var("BENCHMARK_MODE").is_ok() {
                         let json = serde_json::to_string_pretty(&stats).unwrap();
                         std::fs::write("perf_metrics.json", json).unwrap();
                         println!("Benchmark: Exported metrics to perf_metrics.json");
                         event_loop.exit();
                    }
                } else if std::env::var("BENCHMARK_MODE").is_ok() {
                     // Headless export even if render state failed
                     let stats = AppStats {
                         fps: 0,
                         layout_time_micros: 0,
                         node_count: 0,
                         frame_time_micros: 0,
                         bridge_time_micros: 0,
                         render_time_micros: 0,
                         gpu_time_micros: 0,
                         hit_test_time_micros: 0,
                         process_memory_rss_bytes: 0,
                         cpu_usage: 0.0,
                         total_memory: 0,
                         scheduler_iteration: 0,
                         batch_size: 0,
                         text_cache_size: 0,
                         texture_cache_size: 0,
                     };
                     let json = serde_json::to_string_pretty(&stats).unwrap();
                     let _ = std::fs::write("perf_metrics.json", json);
                     println!("Benchmark: Exported fallback metrics to perf_metrics.json");
                     event_loop.exit();
                }

                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            _ => (),
        }
    }
}
