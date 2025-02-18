use std::sync::Arc;

use log::{error, info};
use pollster::block_on;
use wgpu::util::DeviceExt;
use winit::event::KeyEvent;
use winit::window::Window;

use crate::system::{model, resources, texture, vertex};
use crate::system::camera::{Camera, CameraUniform};
use crate::system::camera_controller::CameraController;
use crate::system::model::DrawModel;
use crate::system::shape_geometry::ShapeGeometryFactory;
use crate::system::shapes::{ShapeType, Square};
use crate::system::shapes::Sphere;
use crate::system::transform::{Transform, TransformRaw};

/// 描画を行う構造体
pub struct Renderer<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_caps: wgpu::SurfaceCapabilities,
    pub surface_format: wgpu::TextureFormat,
    pub surface_config: wgpu::SurfaceConfiguration,

    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,

    shape_factory: ShapeGeometryFactory,

    spheres: Vec<Sphere>,
    sphere_transform_buffer: wgpu::Buffer,
    sphere_render_pipeline: wgpu::RenderPipeline,

    obj_model: model::Model,
    obj_model_transforms: Vec<Transform>,
    obj_model_transform_buffer: wgpu::Buffer,
    obj_model_render_pipeline: wgpu::RenderPipeline,

    plane: Square,
    plane_transform_buffer: wgpu::Buffer,
    depth_texture: texture::Texture,
    depth_bind_group_layout: wgpu::BindGroupLayout,
    depth_bind_group: wgpu::BindGroup,
    depth_render_pipeline: wgpu::RenderPipeline,

    instant_time: std::time::Instant,
}

pub const SPHERE: ShapeType = ShapeType::Sphere(16);

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
        let surface_format = surface_caps.formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Immediate,
            view_formats: Vec::new(),
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        let camera = Camera::new(&window);
        let camera_controller = CameraController::new(0.1);

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
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }
                ],
                label: Some("camera_bind_group"),
            }
        );

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                }
            ],
            label: Some("texture_bind_group_layout"),
        });

        let mut shape_factory = ShapeGeometryFactory::new();

        let spheres = (0..10).flat_map(|z| {
            (0..10).map(move |x| {
                let mut sphere = Sphere::new();
                sphere.transform
                    .set_position_x(x as f32 * 2.0)
                    .set_position_z(z as f32 * 2.0);

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

        shape_factory.create_geometry(&device, SPHERE);

        let obj_model = resources::load_model(
            "cube.obj",
            &device,
            &queue,
            &texture_bind_group_layout,
        ).unwrap();

        let obj_model_transforms = (0..10).flat_map(|z| {
            (0..10).map(move |x| {
                let mut transform = Transform::new();
                transform
                    .set_position_x(x as f32 * 4.0 - 18.0)
                    .set_position_z(z as f32 * 4.0 - 40.0)
                    .set_rotation_x(x as f32)
                    .set_rotation_z(z as f32);

                transform
            })
        }).collect::<Vec<_>>();

        let obj_model_transform_data = obj_model_transforms.iter()
            .map(TransformRaw::from)
            .collect::<Vec<_>>();

        let obj_model_transform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Obj Model Transform Buffer"),
                contents: bytemuck::cast_slice(&obj_model_transform_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let mut plane = Square::new();
        plane.transform
            .set_position_y(30.0)
            .set_position_z(10.0)
            .set_scale(50.0);

        let transform_data = vec![TransformRaw::from(&plane.transform)];
        let plane_transform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Plane Transform Buffer"),
                contents: bytemuck::cast_slice(&transform_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        shape_factory.create_geometry(&device, ShapeType::Square);

        let depth_texture = texture::Texture::create_depth_texture(&device, &surface_config, "depth_texture");
        let depth_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("depth_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        count: None,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        visibility: wgpu::ShaderStages::FRAGMENT,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        count: None,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        visibility: wgpu::ShaderStages::FRAGMENT,
                    }
                ],
            }
        );

        let depth_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("depth_bind_group"),
                layout: &depth_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&depth_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&depth_texture.sampler),
                    }
                ],
            }
        );

        let sphere_render_pipeline = Self::create_render_pipeline(
            "sphere",
            &device,
            &[&camera_bind_group_layout],
            Self::create_vertex_state(
                &device.create_shader_module(wgpu::include_wgsl!("../../res/shader/sphere_vertex.wgsl")),
                &[vertex::ModelVertex::desc(), TransformRaw::desc()],
            ),
            Self::create_fragment_state(
                &device.create_shader_module(wgpu::include_wgsl!("../../res/shader/sphere_fragment.wgsl")),
                &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            ),
            Self::create_primitive_state(),
            Self::create_depth_stencil_state(),
        );

        let obj_model_render_pipeline = Self::create_render_pipeline(
            "obj_model",
            &device,
            &[
                &camera_bind_group_layout,
                &texture_bind_group_layout,
            ],
            Self::create_vertex_state(
                &device.create_shader_module(wgpu::include_wgsl!("../../res/shader/model_vertex.wgsl")),
                &[vertex::ModelVertex::desc(), TransformRaw::desc()],
            ),
            Self::create_fragment_state(
                &device.create_shader_module(wgpu::include_wgsl!("../../res/shader/model_fragment.wgsl")),
                &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            ),
            Self::create_primitive_state(),
            Self::create_depth_stencil_state(),
        );

        let depth_render_pipeline = Self::create_render_pipeline(
            "depth",
            &device,
            &[
                &camera_bind_group_layout,
                &depth_bind_group_layout,
            ],
            Self::create_vertex_state(
                &device.create_shader_module(wgpu::include_wgsl!("../../res/shader/depth_vertex.wgsl")),
                &[vertex::ModelVertex::desc(), TransformRaw::desc()],
            ),
            Self::create_fragment_state(
                &device.create_shader_module(wgpu::include_wgsl!("../../res/shader/depth_fragment.wgsl")),
                &[
                    Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })
                ],
            ),
            Self::create_primitive_state(),
            None,
        );

        Self {
            surface,
            device,
            queue,
            surface_caps,
            surface_format,
            surface_config,

            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,

            shape_factory,
            spheres,
            sphere_transform_buffer,
            sphere_render_pipeline,

            obj_model,
            obj_model_transforms,
            obj_model_transform_buffer,
            obj_model_render_pipeline,

            plane,
            plane_transform_buffer,
            depth_texture,
            depth_bind_group_layout,
            depth_bind_group,
            depth_render_pipeline,

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

        self.depth_texture = texture::Texture::create_depth_texture(
            &self.device,
            &self.surface_config,
            "depth_texture",
        );
        self.depth_bind_group = self.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("depth_bind_group"),
                layout: &self.depth_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.depth_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.depth_texture.sampler),
                    }
                ],
            }
        );

        self.camera.aspect = self.surface_config.width as f32 / self.surface_config.height as f32;
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") }
        );

        // sphere render pass
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
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

            render_pass.set_pipeline(&self.sphere_render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            let sphere_geometry_buffers = self.shape_factory.get_geometry(&SPHERE).unwrap();

            render_pass.set_vertex_buffer(0, sphere_geometry_buffers.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.sphere_transform_buffer.slice(..));
            render_pass.set_index_buffer(sphere_geometry_buffers.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(
                0..sphere_geometry_buffers.indices_count,
                0,
                0..self.spheres.len() as _,
            );
        }

        // obj_model render pass
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
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

            render_pass.set_pipeline(&self.obj_model_render_pipeline);
            render_pass.set_vertex_buffer(1, self.obj_model_transform_buffer.slice(..));

            render_pass.draw_model_instanced(
                &self.obj_model,
                0..self.obj_model_transforms.len() as u32,
                &self.camera_bind_group,
            );
        }

        // depth texture render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Depth Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.depth_render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.depth_bind_group, &[]);

            let plane_geometry_buffers = self.shape_factory.get_geometry(&ShapeType::Square).unwrap();

            render_pass.set_vertex_buffer(0, plane_geometry_buffers.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.plane_transform_buffer.slice(..));
            render_pass.set_index_buffer(plane_geometry_buffers.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(
                0..plane_geometry_buffers.indices_count,
                0,
                0..1,
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

        let time = self.instant_time.elapsed().as_secs_f32();

        for (index, sphere) in &mut self.spheres.iter_mut().enumerate() {
            sphere.transform
                .set_position(cgmath::Point3::new(
                    (index % 10) as f32 * 3.0 - 13.5,
                    (time * 4.0 + index as f32).sin() * 10.0,
                    (index / 10) as f32 * 3.0 - 13.5,
                ))
                .set_rotation(cgmath::Euler::new(
                    cgmath::Rad(time * 4.0),
                    cgmath::Rad(time * 1.4),
                    cgmath::Rad(time * 2.6),
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

/// private impl
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
        label: &str,
        device: &wgpu::Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        vertex_state: wgpu::VertexState,
        fragment_state: Option<wgpu::FragmentState>,
        primitive_state: wgpu::PrimitiveState,
        depth_stencil_state: Option<wgpu::DepthStencilState>,
    ) -> wgpu::RenderPipeline {
        let layout_label = format!("{}_render_pipeline_layout", label);
        let render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some(&layout_label),
                bind_group_layouts,
                push_constant_ranges: &[],
            }
        );

        let label = format!("{}_render_pipeline", label);
        device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some(&label),
                layout: Some(&render_pipeline_layout),
                vertex: vertex_state,
                fragment: fragment_state,
                primitive: primitive_state,
                depth_stencil: depth_stencil_state,
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
            module: &shader,
            entry_point: Some("vs_main"),
            buffers,
            compilation_options: Default::default(),
        }
    }

    fn create_fragment_state<'a>(
        shader: &'a wgpu::ShaderModule,
        targets: &'a [Option<wgpu::ColorTargetState>])
        -> Option<wgpu::FragmentState<'a>> {
        Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets,
            compilation_options: Default::default(),
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

    fn create_depth_stencil_state() -> Option<wgpu::DepthStencilState> {
        Some(wgpu::DepthStencilState {
            format: texture::Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        })
    }
}