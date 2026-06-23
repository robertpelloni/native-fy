use std::sync::Arc;
use std::time::{Instant};
use std::collections::HashMap;
use wgpu::util::DeviceExt;
use glyphon::{
    FontSystem, SwashCache, TextAtlas, TextRenderer, TextArea, TextBounds,
    Resolution, Metrics, Family, Shaping,
};
use crate::layout::{LayoutEngine};
use crate::stats::AppStats;

pub const INITIAL_MAX_NODES: usize = 1024;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
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

pub const VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.0, 0.0] },
    Vertex { position: [0.0, 1.0, 0.0] },
    Vertex { position: [1.0, 1.0, 0.0] },
    Vertex { position: [1.0, 0.0, 0.0] },
];

pub const INDICES: &[u16] = &[
    0, 1, 2,
    2, 3, 0,
];

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Globals {
    pub screen_size: [f32; 2],
    pub _padding: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NodeData {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub mode: u32,
    pub _padding: [f32; 3],
}

pub struct RenderState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub globals_buffer: wgpu::Buffer,
    pub node_buffer: wgpu::Buffer,
    pub node_buffer_capacity: usize,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,

    // Text rendering components
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub viewport: glyphon::Viewport,
    pub text_atlas: TextAtlas,
    pub text_renderer: TextRenderer,
    pub text_buffers: HashMap<taffy::prelude::NodeId, (glyphon::Buffer, Instant)>,
    pub stats_buffer: Option<glyphon::Buffer>,
    pub text_eviction_threshold: usize,

    // Textures
    pub textures: HashMap<String, (wgpu::BindGroup, Instant)>,
    pub _diffuse_texture: wgpu::Texture,
    pub texture_eviction_threshold: usize,
}

impl RenderState {
    pub async fn new(window: Arc<winit::window::Window>) -> Result<Self, String> {
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

        let mut usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
        if surface_caps.usages.contains(wgpu::TextureUsages::COPY_SRC) {
            usage |= wgpu::TextureUsages::COPY_SRC;
        }

        let config = wgpu::SurfaceConfiguration {
            usage,
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
            text_eviction_threshold: 200,

            textures: HashMap::new(),
            texture_eviction_threshold: 50,
            _diffuse_texture,
        })
    }

    pub fn capture_frame(&self, texture: &wgpu::Texture, path: String) {
        let size = self.size;
        let u32_size = std::mem::size_of::<u32>() as u32;
        let align = 256;
        let unpadded_bytes_per_row = u32_size * size.width;
        let padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padding;

        let output_buffer_size = (padded_bytes_per_row * size.height) as wgpu::BufferAddress;
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            label: Some("Screenshot Buffer"),
            mapped_at_creation: false,
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Screenshot Encoder"),
        });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(size.height),
                },
            },
            wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        let buffer_slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| tx.send(v).unwrap());
        self.device.poll(wgpu::Maintain::Wait);

        if let Ok(Ok(())) = rx.recv() {
            let padded_data = buffer_slice.get_mapped_range();
            let mut data = Vec::with_capacity((unpadded_bytes_per_row * size.height) as usize);

            for chunk in padded_data.chunks(padded_bytes_per_row as usize) {
                data.extend_from_slice(&chunk[..unpadded_bytes_per_row as usize]);
            }

            // Handle Bgra8UnormSrgb to Rgba8 conversion if necessary
            if self.config.format == wgpu::TextureFormat::Bgra8UnormSrgb || self.config.format == wgpu::TextureFormat::Bgra8Unorm {
                for chunk in data.chunks_exact_mut(4) {
                    chunk.swap(0, 2);
                }
            }

            if let Some(img) = image::RgbaImage::from_raw(size.width, size.height, data) {
                let _ = img.save(path);
            }
            drop(padded_data);
            output_buffer.unmap();
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

    pub fn render_dashboard(&mut self, stats: &AppStats, node_count: u32, screenshot_path: Option<String>) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Update stats text
        let version = env!("CARGO_PKG_VERSION");
        let stats_text = format!(
            "MONITORING DASHBOARD | v{} | Status: HEALTHY | FPS: {} | CPU: {:.1}% | Mem: {}MB | Bridge: {}µs | Layout: {}µs | Render: {}µs | Nodes: {} | Iter: {} | Batch: {} | Cache(T/TX): {}/{}",
            version, stats.fps, stats.cpu_usage, stats.process_memory_rss_bytes / 1024 / 1024, stats.bridge_time_micros, stats.layout_time_micros, stats.render_time_micros, stats.node_count, stats.scheduler_iteration, stats.batch_size, stats.text_cache_size, stats.texture_cache_size
        );
        let stats_buffer = self.stats_buffer.get_or_insert_with(|| {
            glyphon::Buffer::new(&mut self.font_system, Metrics::new(12.0, 16.0))
        });
        stats_buffer.set_size(&mut self.font_system, Some(self.size.width as f32), Some(20.0));
        stats_buffer.set_text(&mut self.font_system, &stats_text, glyphon::Attrs::new().family(Family::Monospace).color(glyphon::Color::rgb(0, 255, 0)), Shaping::Advanced);
        stats_buffer.shape_until_scroll(&mut self.font_system, false);

        let text_areas = vec![TextArea {
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
        }];

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
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..node_count);

            self.text_renderer.render(&self.text_atlas, &self.viewport, &mut render_pass).unwrap();
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        if let Some(path) = screenshot_path {
            self.capture_frame(&output.texture, path);
        }
        output.present();
        Ok(())
    }

    pub fn render(&mut self, engine: &LayoutEngine, root_id: taffy::prelude::NodeId, stats: &AppStats, screenshot_path: Option<String>) -> Result<(), wgpu::SurfaceError> {
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

        // Update Text Buffers with LRU Eviction Policy
        if self.text_buffers.len() > self.text_eviction_threshold {
            let mut entries: Vec<_> = self.text_buffers.iter().map(|(k, v)| (*k, v.1)).collect();
            entries.sort_by_key(|&(_, last_used)| last_used);
            let evict_count = (self.text_eviction_threshold / 4).max(1);
            for i in 0..evict_count.min(entries.len()) {
                self.text_buffers.remove(&entries[i].0);
            }
            #[cfg(debug_assertions)]
            if !std::env::var("PROD_MODE").is_ok() {
                println!("Memory: Evicted {} text buffers (LRU).", evict_count);
            }
        }
        for (id, text, _, _, width, height) in &text_data {
            let (buffer, last_used) = self.text_buffers.entry(*id).or_insert_with(|| {
                (glyphon::Buffer::new(&mut self.font_system, Metrics::new(16.0, 20.0)), Instant::now())
            });
            *last_used = Instant::now();
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
        let version = env!("CARGO_PKG_VERSION");
        let stats_text = if std::env::var("PROD_MODE").is_ok() {
            String::new()
        } else if std::env::var("DASHBOARD_MODE").is_ok() {
            format!(
                "MONITORING DASHBOARD | v{} | Status: HEALTHY | FPS: {} | Bridge: {}µs | Layout: {}µs | Render: {}µs | Nodes: {}",
                version, stats.fps, stats.bridge_time_micros, stats.layout_time_micros, stats.render_time_micros, stats.node_count
            )
        } else {
            format!(
                "v{} | FPS: {} | Layout: {}µs | Nodes: {} | Protocol: ACTIVE (AUTO-SYNC)",
                version, stats.fps, stats.layout_time_micros, stats.node_count
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
            if let Some((buffer, _)) = self.text_buffers.get(id) {
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
                        if let Some((bg, last_used)) = self.textures.get_mut(url) {
                            *last_used = Instant::now();
                            bg
                        } else {
                            &self.bind_group
                        }
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

        if let Some(path) = screenshot_path {
            self.capture_frame(&output.texture, path);
        }

        output.present();

        let _render_duration = render_start.elapsed();
        // Skip per-frame console logging in production/performance runs
        #[cfg(debug_assertions)]
        if !std::env::var("PROD_MODE").is_ok() {
             println!("Performance: Frame rendered in {:?}", _render_duration);
        }

        Ok(())
    }

    pub fn collect_nodes(
        &mut self,
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
            let is_svg = node_type == Some(&"Svg".to_string());

            let mut texture_url = None;

            if is_image {
                texture_url = engine.get_value(id).cloned();
            } else if is_svg {
                if let Some(svg_content) = engine.get_value(id) {
                    let cache_key = format!("svg:{:?}", id);
                    if !self.textures.contains_key(&cache_key) {
                        // Render SVG to texture
                        if let Some(rgba) = self.render_svg_to_rgba(svg_content, layout.size.width, layout.size.height) {
                            self.upload_texture(cache_key.clone(), &rgba, layout.size.width as u32, layout.size.height as u32);
                        }
                    }
                    texture_url = Some(cache_key);
                }
            }

            nodes.push(NodeData {
                pos: [x, y],
                size: [layout.size.width, layout.size.height],
                color: if is_image || is_svg { [1.0, 1.0, 1.0, 1.0] } else { [0.5, 0.6, 0.7, 1.0] },
                mode: if is_image || is_svg { 1 } else { 0 },
                _padding: [0.0; 3],
            });

            node_textures.push(texture_url);

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

    fn render_svg_to_rgba(&self, svg_content: &str, width: f32, height: f32) -> Option<Vec<u8>> {
        let opt = usvg::Options::default();
        let rtree = usvg::Tree::from_str(svg_content, &opt).ok()?;

        let mut pixmap = tiny_skia::Pixmap::new(width as u32, height as u32)?;
        resvg::render(&rtree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

        Some(pixmap.data().to_vec())
    }

    fn upload_texture(&mut self, url: String, rgba: &[u8], width: u32, height: u32) {
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&url),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
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
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        if self.textures.len() > self.texture_eviction_threshold {
            let mut entries: Vec<_> = self.textures.iter().map(|(k, v)| (k.clone(), v.1)).collect();
            entries.sort_by_key(|&(_, last_used)| last_used);
            let evict_count = (self.texture_eviction_threshold / 5).max(1);
            for i in 0..evict_count.min(entries.len()) {
                self.textures.remove(&entries[i].0);
            }
        }
        self.textures.insert(url, (bind_group, Instant::now()));
    }
}
