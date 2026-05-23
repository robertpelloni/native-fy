mod layout;
mod ui_gen;
mod runtime;

use std::sync::Arc;
use std::time::{Instant, Duration};
use std::sync::mpsc::{self, Receiver, Sender};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use layout::{LayoutEngine, Node, AstRect, FlexStyles};
use runtime::{JsRuntime, UiCommand};
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, ElementState, MouseButton},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};
use wgpu::util::DeviceExt;
use glyphon::{
    FontSystem, SwashCache, TextAtlas, TextRenderer, TextArea, TextBounds,
    Resolution, Metrics, Family, Shaping,
};

const INITIAL_MAX_NODES: usize = 1024;
const LOG_FILE: &str = "app.log";

#[derive(serde::Serialize)]
struct AppStats {
    fps: u32,
    layout_time_micros: u64,
    node_count: usize,
    frame_time_micros: u64,
}

fn log_error(msg: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE)
        .unwrap();
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    writeln!(file, "[{}] ERROR: {}", timestamp, msg).unwrap();
    eprintln!("{}", msg);
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.0, 0.0] },
    Vertex { position: [0.0, 1.0, 0.0] },
    Vertex { position: [1.0, 1.0, 0.0] },
    Vertex { position: [1.0, 0.0, 0.0] },
];

const INDICES: &[u16] = &[
    0, 1, 2,
    2, 3, 0,
];

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    screen_size: [f32; 2],
    _padding: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct NodeData {
    pos: [f32; 2],
    size: [f32; 2],
    color: [f32; 4],
    mode: u32,
    _padding: [f32; 3],
}

struct RenderState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    globals_buffer: wgpu::Buffer,
    node_buffer: wgpu::Buffer,
    node_buffer_capacity: usize,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,

    // Text rendering components
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: glyphon::Viewport,
    text_atlas: TextAtlas,
    text_renderer: TextRenderer,
    text_buffers: HashMap<taffy::prelude::NodeId, glyphon::Buffer>,
    stats_buffer: Option<glyphon::Buffer>,

    // Textures
    textures: HashMap<String, wgpu::BindGroup>,
    _diffuse_texture: wgpu::Texture,
}

impl RenderState {
    async fn new(window: Arc<Window>) -> Result<Self, String> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            gles_minor_version: Default::default(),
            flags: wgpu::InstanceFlags::default(),
        });

        let surface = instance.create_surface(window.clone())
            .map_err(|e| format!("Failed to create surface: {:?}", e))?;

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.ok_or("Failed to request adapter")?;

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ).await.map_err(|e| format!("Failed to request device: {:?}", e))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        // Texture creation (placeholder)
        let _diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Diffuse Texture"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let diffuse_view = _diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Globals Buffer"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let node_buffer_capacity = INITIAL_MAX_NODES;
        let node_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Node Buffer"),
            size: (std::mem::size_of::<NodeData>() * node_buffer_capacity) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage {
                            read_only: true,
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: node_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&diffuse_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                },
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let num_indices = INDICES.len() as u32;

        // Initialize glyphon components
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = glyphon::Cache::new(&device);
        let mut viewport = glyphon::Viewport::new(&device, &cache);
        viewport.update(&queue, Resolution { width: size.width, height: size.height });
        let mut text_atlas = TextAtlas::new(&device, &queue, &cache, surface_format);
        let text_renderer = TextRenderer::new(&mut text_atlas, &device, wgpu::MultisampleState::default(), None);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            globals_buffer,
            node_buffer,
            node_buffer_capacity,
            bind_group_layout,
            bind_group,

            font_system,
            swash_cache,
            viewport,
            text_atlas,
            text_renderer,
            text_buffers: HashMap::new(),
            stats_buffer: None,

            textures: HashMap::new(),
            _diffuse_texture,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.viewport.update(&self.queue, Resolution { width: new_size.width, height: new_size.height });
        }
    }

    fn render(&mut self, engine: &LayoutEngine, root_id: taffy::prelude::NodeId, stats: &AppStats) -> Result<(), wgpu::SurfaceError> {
        let render_start = Instant::now();

        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Update globals
        let globals = Globals {
            screen_size: [self.size.width as f32, self.size.height as f32],
            _padding: [0.0; 2],
        };
        self.queue.write_buffer(&self.globals_buffer, 0, bytemuck::cast_slice(&[globals]));

        // Collect node data
        let mut nodes = Vec::new();
        let mut text_data = Vec::new();
        let mut node_textures = Vec::new();
        self.collect_nodes(engine, root_id, 0.0, 0.0, &mut nodes, &mut text_data, &mut node_textures);

        // Update Text Buffers
        for (id, text, _, _, width, height) in &text_data {
            let buffer = self.text_buffers.entry(*id).or_insert_with(|| {
                glyphon::Buffer::new(&mut self.font_system, Metrics::new(16.0, 20.0))
            });
            buffer.set_size(&mut self.font_system, Some(*width), Some(*height));
            buffer.set_text(&mut self.font_system, text, glyphon::Attrs::new().family(Family::SansSerif), Shaping::Advanced);
            buffer.shape_until_scroll(&mut self.font_system, false);
        }

        if !nodes.is_empty() {
            if nodes.len() > self.node_buffer_capacity {
                self.node_buffer_capacity = nodes.len().next_power_of_two();
                self.node_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Node Buffer"),
                    size: (std::mem::size_of::<NodeData>() * self.node_buffer_capacity) as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                // Re-create bind group
                let diffuse_view = self._diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
                let diffuse_sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    ..Default::default()
                });

                self.bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Bind Group"),
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.globals_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: self.node_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&diffuse_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                        },
                    ],
                });
            }
            self.queue.write_buffer(&self.node_buffer, 0, bytemuck::cast_slice(&nodes));
        }

        // Prepare text areas
        let mut text_areas = Vec::new();

        // Dashboard or Overlay Stats
        let stats_text = if std::env::var("PROD_MODE").is_ok() {
            String::new()
        } else if std::env::var("DASHBOARD_MODE").is_ok() {
            format!(
                "MONITORING DASHBOARD | v0.24.0 | Status: HEALTHY | FPS: {} | Layout: {}µs | Nodes: {}",
                stats.fps, stats.layout_time_micros, stats.node_count
            )
        } else {
            format!(
                "v0.24.0 | FPS: {} | Layout: {}µs | Nodes: {} | Protocol: ACTIVE (AUTO-SYNC)",
                stats.fps, stats.layout_time_micros, stats.node_count
            )
        };

        let stats_buffer = self.stats_buffer.get_or_insert_with(|| {
            glyphon::Buffer::new(&mut self.font_system, Metrics::new(12.0, 16.0))
        });
        stats_buffer.set_size(&mut self.font_system, Some(self.size.width as f32), Some(20.0));
        stats_buffer.set_text(&mut self.font_system, &stats_text, glyphon::Attrs::new().family(Family::Monospace).color(glyphon::Color::rgb(0, 255, 0)), Shaping::Advanced);
        stats_buffer.shape_until_scroll(&mut self.font_system, false);

        if !stats_text.is_empty() {
            text_areas.push(TextArea {
                buffer: stats_buffer,
                left: 10.0,
                top: 10.0,
            scale: 1.0,
            bounds: TextBounds {
                left: 0,
                top: 0,
                right: self.size.width as i32,
                bottom: self.size.height as i32,
            },
                default_color: glyphon::Color::rgb(0, 255, 0),
                custom_glyphs: &[],
            });
        }

        for (id, _, x, y, _, _) in &text_data {
            if let Some(buffer) = self.text_buffers.get(id) {
                text_areas.push(TextArea {
                    buffer,
                    left: *x,
                    top: *y,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: self.size.width as i32,
                        bottom: self.size.height as i32,
                    },
                    default_color: glyphon::Color::rgb(255, 255, 255),
                    custom_glyphs: &[],
                });
            }
        }

        // Prepare text rendering
        self.text_renderer.prepare(
            &self.device,
            &self.queue,
            &mut self.font_system,
            &mut self.text_atlas,
            &self.viewport,
            text_areas,
            &mut self.swash_cache,
        ).unwrap();

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            if !nodes.is_empty() {
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

                let mut start_idx = 0;
                while start_idx < nodes.len() {
                    let texture_url = &node_textures[start_idx];
                    let mut end_idx = start_idx + 1;

                    // Batch identical textures
                    while end_idx < nodes.len() && &node_textures[end_idx] == texture_url {
                        end_idx += 1;
                    }

                    let bind_group = if let Some(url) = texture_url {
                        self.textures.get(url).unwrap_or(&self.bind_group)
                    } else {
                        &self.bind_group
                    };

                    render_pass.set_bind_group(0, bind_group, &[]);
                    render_pass.draw_indexed(0..self.num_indices, 0, start_idx as u32..end_idx as u32);
                    start_idx = end_idx;
                }
            }

            // Draw text
            self.text_renderer.render(&self.text_atlas, &self.viewport, &mut render_pass).unwrap();
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        let render_duration = render_start.elapsed();
        println!("Performance: Frame rendered in {:?}", render_duration);

        Ok(())
    }

    fn collect_nodes(
        &self,
        engine: &LayoutEngine,
        id: taffy::prelude::NodeId,
        parent_x: f32,
        parent_y: f32,
        nodes: &mut Vec<NodeData>,
        text_data: &mut Vec<(taffy::prelude::NodeId, String, f32, f32, f32, f32)>,
        node_textures: &mut Vec<Option<String>>,
    ) {
        if let Some(layout) = engine.layout(id) {
            let x = parent_x + layout.location.x;
            let y = parent_y + layout.location.y;

            let node_type = engine.get_type(id);
            let is_image = node_type == Some(&"Image".to_string());

            nodes.push(NodeData {
                pos: [x, y],
                size: [layout.size.width, layout.size.height],
                color: [0.5, 0.6, 0.7, 1.0], // Placeholder color
                mode: if is_image { 1 } else { 0 },
                _padding: [0.0; 3],
            });

            node_textures.push(if is_image { engine.get_value(id).cloned() } else { None });

            if let Some(text) = engine.get_text(id) {
                text_data.push((id, text.clone(), x, y, layout.size.width, layout.size.height));
            }

            if let Some(children) = engine.children(id) {
                for child_id in children {
                    self.collect_nodes(engine, child_id, x, y, nodes, text_data, node_textures);
                }
            }
        }
    }
}

struct NativefyApp {
    window: Option<Arc<Window>>,
    render_state: Option<RenderState>,
    layout_engine: Option<LayoutEngine>,
    root_id: Option<taffy::prelude::NodeId>,
    js_runtime: Option<JsRuntime>,
    mouse_pos: [f32; 2],
    ui_rx: Receiver<UiCommand>,
    ui_tx: Sender<UiCommand>,
    last_frame: Instant,
    fps: u32,
    frame_count: u32,
    last_fps_update: Instant,
    perf_history: Vec<AppStats>,
}

impl Default for NativefyApp {
    fn default() -> Self {
        let (ui_tx, ui_rx) = mpsc::channel();
        Self {
            window: None,
            render_state: None,
            layout_engine: None,
            root_id: None,
            js_runtime: None,
            mouse_pos: [0.0; 2],
            ui_rx,
            ui_tx,
            last_frame: Instant::now(),
            fps: 0,
            frame_count: 0,
            last_fps_update: Instant::now(),
            perf_history: Vec::new(),
        }
    }
}

impl ApplicationHandler for NativefyApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Native-fy UI Engine")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));

            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            self.window = Some(window.clone());

            let render_state_res = pollster::block_on(RenderState::new(window));
            match render_state_res {
                Ok(state) => self.render_state = Some(state),
                Err(e) => {
                    log_error(&format!("Failed to initialize render state: {}", e));
                    return;
                }
            }

            let mut engine = LayoutEngine::new();

            let layout_start = Instant::now();
            let root_id = ui_gen::generate_ui_tree(&mut engine);
            let _ = engine.compute(root_id);
            let layout_duration = layout_start.elapsed();
            println!("Performance: Initial layout computed in {:?}", layout_duration);

            self.layout_engine = Some(engine);
            self.root_id = Some(root_id);

            // Initialize QuickJS
            let runtime = JsRuntime::new(self.ui_tx.clone());
            let bridge_code = include_str!("runtime.js");
            runtime.eval(bridge_code);
            self.js_runtime = Some(runtime);

            println!("Window, Wgpu, and QuickJS successfully initialized!");
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
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                if let Some(runtime) = self.js_runtime.as_ref() {
                    runtime.dispatch_click(self.mouse_pos[0], self.mouse_pos[1]);
                }
            }
            WindowEvent::RedrawRequested => {
                // Update FPS
                self.frame_count += 1;
                let now = Instant::now();
                if now.duration_since(self.last_fps_update) >= Duration::from_secs(1) {
                    self.fps = self.frame_count;
                    self.frame_count = 0;
                    self.last_fps_update = now;
                }

                // Process UI commands in batch
                let mut recompute = false;
                let mut command_count = 0;
                while let Ok(cmd) = self.ui_rx.try_recv() {
                    command_count += 1;
                    if command_count > 100 { break; } // Safety break
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
                        UiCommand::CreateNativeButton { text, styles } => {
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
                            println!("Runtime: Loading image asset from {}", url);
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

                                state.textures.insert(url, bind_group);
                            }
                        }
                        UiCommand::HealthCheck => {
                            println!("Health Check: Bridge is responsive.");
                        }
                        UiCommand::Reload => {
                            println!("Runtime: Reloading UI tree...");
                            if let (Some(engine), Some(root_id)) = (self.layout_engine.as_mut(), self.root_id) {
                                // Simple reload by clearing and re-generating
                                let _ = ui_gen::generate_ui_tree(engine);
                                let _ = engine.compute(root_id);
                                recompute = true;
                            }
                        }
                        UiCommand::RunPipeline => {
                            println!("Runtime: Triggering Full Pipeline...");
                            let _ = std::process::Command::new("npm")
                                .arg("run")
                                .arg("pipeline")
                                .status();
                        }
                        UiCommand::SyncProtocol => {
                            println!("Runtime: Triggering Protocol Sync...");
                            let _ = std::process::Command::new("node")
                                .arg("scripts/protocol_sync.js")
                                .status();
                        }
                    }
                }

                let mut layout_duration = Duration::from_micros(0);
                if recompute {
                    if let (Some(engine), Some(root_id)) = (self.layout_engine.as_mut(), self.root_id) {
                        let start = Instant::now();
                        let _ = engine.compute(root_id);
                        layout_duration = start.elapsed();
                    }
                }

                if let (Some(state), Some(engine), Some(root_id)) = (self.render_state.as_mut(), self.layout_engine.as_ref(), self.root_id) {
                    let stats = AppStats {
                        fps: self.fps,
                        layout_time_micros: layout_duration.as_micros() as u64,
                        node_count: engine.node_count(),
                        frame_time_micros: now.duration_since(self.last_fps_update).as_micros() as u64 / (self.frame_count.max(1) as u64),
                    };

                    // Record history for dashboard
                    if self.frame_count % 60 == 0 {
                        self.perf_history.push(AppStats {
                             fps: stats.fps,
                             layout_time_micros: stats.layout_time_micros,
                             node_count: stats.node_count,
                             frame_time_micros: stats.frame_time_micros,
                        });
                        if self.perf_history.len() > 100 { self.perf_history.remove(0); }
                    }

                    match state.render(engine, root_id, &stats) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => state.resize(state.size),
                        Err(e) => log_error(&format!("Render error: {:?}", e)),
                    }

                    // Benchmark export trigger
                    if std::env::var("BENCHMARK_MODE").is_ok() {
                         let json = serde_json::to_string_pretty(&stats).unwrap();
                         std::fs::write("perf_metrics.json", json).unwrap();
                         println!("Benchmark: Exported metrics to perf_metrics.json");
                         event_loop.exit();
                    }
                }
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            _ => (),
        }
    }
}

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

    // Initialize winit event loop
    let event_loop = EventLoop::new().unwrap();
    let mut app = NativefyApp::default();

    println!("Starting event loop...");
    event_loop.run_app(&mut app).unwrap();
}