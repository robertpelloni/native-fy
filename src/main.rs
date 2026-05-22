mod layout;
mod ui_gen;
mod runtime;

use std::sync::Arc;
use std::time::Instant;
use std::sync::mpsc::{self, Receiver, Sender};
use layout::LayoutEngine;
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

const MAX_NODES: usize = 1024;

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
    bind_group: wgpu::BindGroup,

    // Text rendering components
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: glyphon::Viewport,
    text_atlas: TextAtlas,
    text_renderer: TextRenderer,
}

impl RenderState {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            gles_minor_version: Default::default(),
            flags: wgpu::InstanceFlags::default(),
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ).await.unwrap();

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

        let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Globals Buffer"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let node_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Node Buffer"),
            size: (std::mem::size_of::<NodeData>() * MAX_NODES) as u64,
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

        Self {
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
            bind_group,

            font_system,
            swash_cache,
            viewport,
            text_atlas,
            text_renderer,
        }
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

    fn render(&mut self, engine: &LayoutEngine, root_id: taffy::prelude::NodeId) -> Result<(), wgpu::SurfaceStatus> {
        let render_start = Instant::now();

        let output = self.surface.get_current_texture();
        let output = match output {
            Ok(texture) => texture,
            Err(wgpu::SurfaceError::Timeout) => return Err(wgpu::SurfaceStatus::Timeout),
            Err(wgpu::SurfaceError::Outdated) => return Err(wgpu::SurfaceStatus::Outdated),
            Err(wgpu::SurfaceError::Lost) => return Err(wgpu::SurfaceStatus::Lost),
            Err(wgpu::SurfaceError::OutOfMemory) => return Err(wgpu::SurfaceStatus::Lost), // Simplified
        };

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
        self.collect_nodes(engine, root_id, 0.0, 0.0, &mut nodes, &mut text_data);

        if !nodes.is_empty() {
            let data_to_write = if nodes.len() > MAX_NODES {
                &nodes[..MAX_NODES]
            } else {
                &nodes
            };
            self.queue.write_buffer(&self.node_buffer, 0, bytemuck::cast_slice(data_to_write));
        }

        // Prepare text areas
        let mut text_areas = Vec::new();
        let mut buffers = Vec::new(); // Keep buffers alive
        for (text, _x, _y, width, height) in &text_data {
            let mut buffer = glyphon::Buffer::new(&mut self.font_system, Metrics::new(16.0, 20.0));
            buffer.set_size(&mut self.font_system, Some(*width), Some(*height));
            buffer.set_text(&mut self.font_system, text, glyphon::Attrs::new().family(Family::SansSerif), Shaping::Advanced);
            buffer.shape_until_scroll(&mut self.font_system, false);
            buffers.push(buffer);
        }

        for (i, (_, x, y, _, _)) in text_data.iter().enumerate() {
            text_areas.push(TextArea {
                buffer: &buffers[i],
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
                let instance_count = nodes.len().min(MAX_NODES) as u32;
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.num_indices, 0, 0..instance_count);
            }

            // Draw text
            self.text_renderer.render(&self.text_atlas, &self.viewport, &mut render_pass).unwrap();
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        let render_duration = render_start.elapsed();
        // println!("Performance: Frame rendered in {:?}", render_duration);

        Ok(())
    }

    fn collect_nodes(
        &self,
        engine: &LayoutEngine,
        id: taffy::prelude::NodeId,
        parent_x: f32,
        parent_y: f32,
        nodes: &mut Vec<NodeData>,
        text_data: &mut Vec<(String, f32, f32, f32, f32)>,
    ) {
        if let Some(layout) = engine.layout(id) {
            let x = parent_x + layout.location.x;
            let y = parent_y + layout.location.y;

            nodes.push(NodeData {
                pos: [x, y],
                size: [layout.size.width, layout.size.height],
                color: [0.5, 0.6, 0.7, 1.0], // Placeholder color
            });

            if let Some(text) = engine.get_text(id) {
                text_data.push((text.clone(), x, y, layout.size.width, layout.size.height));
            }

            if let Some(children) = engine.children(id) {
                for child_id in children {
                    self.collect_nodes(engine, child_id, x, y, nodes, text_data);
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

            let render_state = pollster::block_on(RenderState::new(window));
            self.render_state = Some(render_state);

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
                // Process UI commands
                while let Ok(cmd) = self.ui_rx.try_recv() {
                    match cmd {
                        UiCommand::CreateNode { node_type, styles, text } => {
                            println!("Native: Handling CreateNode for {}", node_type);
                            // Full implementation would add to LayoutEngine and recompute
                        }
                    }
                }

                if let (Some(state), Some(engine), Some(root_id)) = (self.render_state.as_mut(), self.layout_engine.as_ref(), self.root_id) {
                    match state.render(engine, root_id) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceStatus::Lost) => state.resize(state.size),
                        Err(e) => eprintln!("{:?}", e),
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
    println!("Initializing Native-fy Windowing Environment...");

    // Initialize winit event loop
    let event_loop = EventLoop::new().unwrap();
    let mut app = NativefyApp::default();

    println!("Starting event loop...");
    event_loop.run_app(&mut app).unwrap();
}