use std::future::Future;
use std::ops::Sub;
use std::sync::Arc;

use image::GenericImageView;
use log::{debug, error, info};
use pollster::block_on;
use wgpu::util::{DeviceExt, RenderEncoder};
use wgpu::WasmNotSend;
use winit::dpi::PhysicalSize;
use winit::event::KeyEvent;
use winit::window::Window;

use crate::system::game_object::transform::TransformRaw;
use crate::system::primitive_shapes;
use crate::system::primitive_shapes::sphere::Sphere;
use crate::system::rendering::{camera, vertex};
use crate::system::rendering::camera::{Camera, CameraUniform};
use crate::system::rendering::camera_controller;
use crate::system::rendering::camera_controller::CameraController;
use crate::system::rendering::shape_geometry_buffers::ShapeGeometryBuffers;

/// 描画を行う構造体
pub struct Renderer<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_caps: wgpu::SurfaceCapabilities,
    pub surface_format: wgpu::TextureFormat,
    pub render_pipeline: wgpu::RenderPipeline,
    pub camera_controller: CameraController,
    sphere_geometry_buffers: ShapeGeometryBuffers,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    spheres: Vec<Sphere>,
    sphere_transform_buffer: wgpu::Buffer,
    instant_time: std::time::Instant,
}

impl<'a> Renderer<'a> {
    pub fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::new(
            &wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                ..Default::default()
            });

        let surface = match instance.create_surface(Arc::clone(&window)) {
            Ok(surface) => surface,
            Err(e) => {
                error!("Failed to create surface: {:?}", e);
                std::process::exit(1);
            }
        };

        let adapter = Self::create_adapter(&instance, &surface);

        let (device, queue) = Self::create_device_and_queue(&adapter);

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);

        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: window.inner_size().width,
                height: window.inner_size().height,
                present_mode: wgpu::PresentMode::Immediate,
                view_formats: Vec::new(),
                alpha_mode: surface_caps.alpha_modes[0],
                desired_maximum_frame_latency: 2,
            },
        );

        let camera = Camera::new(&window);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
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
                    }
                ],
                label: Some("camera_bind_group_layout"),
            }
        );

        let camera_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &camera_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: camera_buffer.as_entire_binding() }
                ],
                label: Some("camera_bind_group"),
            }
        );

        let camera_controller = CameraController::new(0.1);

        let spheres = (0..10).flat_map(|z| {
            (0..10).map(move |x| {
                let mut sphere = Sphere::new(1.0);
                sphere.transform.set_position(cgmath::Point3::new(x as f32 * 2.0, 0.0, z as f32 * 2.0));

                sphere
            })
        }).collect::<Vec<_>>();

        let transform_data = spheres.iter()
            .map(|x| &x.transform)
            .map(TransformRaw::from)
            .collect::<Vec<_>>();

        let sphere_transform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Sphere Transform Buffer"),
                contents: bytemuck::cast_slice(&transform_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let sphere_geometry = Sphere::create_geometry(16);
        let sphere_geometry_buffers = ShapeGeometryBuffers::from(&device, &sphere_geometry);

        let render_pipeline = Self::create_render_pipeline(
            &device,
            &camera_bind_group_layout,
            surface_format,
        );

        Self {
            surface,
            device,
            queue,
            surface_caps,
            surface_format,
            render_pipeline,
            sphere_geometry_buffers,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            spheres,
            sphere_transform_buffer,
            instant_time: std::time::Instant::now(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.surface_format,
                width,
                height,
                present_mode: self.surface_caps.present_modes[0],
                view_formats: Vec::new(),
                alpha_mode: self.surface_caps.alpha_modes[0],
                desired_maximum_frame_latency: 2,
            },
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") }
        );

        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.sphere_geometry_buffers.vertex_buffer.slice(..));

            render_pass.set_vertex_buffer(1, self.sphere_transform_buffer.slice(..));
            render_pass.set_index_buffer(self.sphere_geometry_buffers.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.draw_indexed(
                0..self.sphere_geometry_buffers.indices_count,
                0,
                0..self.spheres.len() as _,
            );
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn key_input(&mut self, key_event: &KeyEvent) {
        self.camera_controller.process_events(key_event);
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);

        let time = self.instant_time.elapsed().as_secs_f32() * 2.0;

        for (index, sphere) in &mut self.spheres.iter_mut().enumerate() {
            sphere.transform.set_rotation(cgmath::Euler::new(
                cgmath::Rad(time * 2.0),
                cgmath::Rad(time * 0.7),
                cgmath::Rad(time * 1.3),
            ));
        }

        let transform_data = self.spheres.iter()
            .map(|x| &x.transform)
            .map(TransformRaw::from)
            .collect::<Vec<_>>();

        self.sphere_transform_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Sphere Transform Buffer"),
                contents: bytemuck::cast_slice(&transform_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
}

/// private implementation
impl Renderer<'_> {
    fn create_adapter(instance: &wgpu::Instance, surface: &wgpu::Surface) -> wgpu::Adapter {
        let request = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }
        );

        match block_on(request) {
            Some(adapter) => {
                info!("Adapter: {:?}", adapter.get_info());
                adapter
            }
            None => {
                error!("Failed to find a suitable adapter");
                std::process::exit(1);
            }
        }
    }

    fn create_device_and_queue(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
        let request = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        );

        match block_on(request) {
            Ok(n) => n,
            Err(e) => {
                error!("Failed to create device and queue: {:?}", e);
                std::process::exit(1);
            }
        }
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let vs_shader = device.create_shader_module(wgpu::include_wgsl!("../../../shader/sphere_vertex.wgsl"));
        let fs_shader = device.create_shader_module(wgpu::include_wgsl!("../../../shader/sphere_fragment.wgsl"));

        let render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                ],
                push_constant_ranges: &[],
            }
        );

        device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: Self::create_vertex_state(
                    &vs_shader,
                    &[vertex::Vertex::desc(), TransformRaw::desc()],
                ),
                fragment: Self::create_fragment_state(
                    &fs_shader,
                    &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })]),
                primitive: Self::create_primitive_state(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            }
        )
    }

    fn create_vertex_state<'a>(
        shader: &'a wgpu::ShaderModule,
        buffers: &'a [wgpu::VertexBufferLayout])
        -> wgpu::VertexState<'a> {
        wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }
    }

    fn create_fragment_state<'a>(
        shader: &'a wgpu::ShaderModule,
        targets: &'a [Option<wgpu::ColorTargetState>])
        -> Option<wgpu::FragmentState<'a>> {
        Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        })
    }

    fn create_primitive_state() -> wgpu::PrimitiveState {
        return wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        };
    }
}