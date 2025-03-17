use std::sync::Arc;

use cgmath::Rotation3;
use log::{error, info};
use pollster::block_on;
use wgpu::util::DeviceExt;
use winit::event::KeyEvent;
use winit::window::Window;

use crate::system::camera::{Camera, CameraUniform};
use crate::system::camera_controller::CameraController;
use crate::system::shape_geometry::ShapeGeometryFactory;
use crate::system::shapes::Sphere;
use crate::system::shapes::{ShapeType, Square};
use crate::system::transform::{Transform, TransformRaw};
use crate::system::{light, obj_model, resources, texture, vertex};

use super::obj_model::DrawLight;
use super::pmx_model;

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

    light_uniform: light::LightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    light_render_pipeline: wgpu::RenderPipeline,

    shape_factory: ShapeGeometryFactory,

    spheres: Vec<Sphere>,
    sphere_transform_buffer: wgpu::Buffer,
    sphere_render_pipeline: wgpu::RenderPipeline,

    obj_model: obj_model::ObjModel,
    debug_material: obj_model::ObjMaterial,
    obj_model_transforms: Vec<Transform>,
    obj_model_transform_buffer: wgpu::Buffer,
    obj_model_render_pipeline: wgpu::RenderPipeline,

    pmx_lumine_model: pmx_model::PMXModel,
    pmx_lumine_model_transform: Transform,
    pmx_lumine_model_transform_buffer: wgpu::Buffer,
    pmx_lumine_model_render_pipeline: wgpu::RenderPipeline,

    pmx_barbara_model: pmx_model::PMXModel,
    pmx_barbara_model_transform: Transform,
    pmx_barbara_model_transform_buffer: wgpu::Buffer,
    pmx_barbara_model_render_pipeline: wgpu::RenderPipeline,

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
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
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

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let light_uniform = crate::system::light::LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0.0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0.0,
        };

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
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
                label: Some("Obj Model Texture Bind Group Layout"),
            });

        let mut shape_factory = ShapeGeometryFactory::new();

        let spheres = (0..10)
            .flat_map(|z| {
                (0..10).map(move |x| {
                    let mut sphere = Sphere::new();
                    sphere
                        .transform
                        .set_position_x(x as f32 * 2.0)
                        .set_position_z(z as f32 * 2.0);

                    sphere
                })
            })
            .collect::<Vec<_>>();

        let transform_data = spheres
            .iter()
            .map(|x| &x.transform)
            .map(TransformRaw::from)
            .collect::<Vec<_>>();

        let sphere_transform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Sphere Transform Buffer"),
                contents: bytemuck::cast_slice(&transform_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

        shape_factory.create_geometry(&device, SPHERE);

        let obj_model =
            resources::load_obj_model("cube.obj", &device, &queue, &texture_bind_group_layout)
                .unwrap();

        let debug_material = {
            let diffuse_bytes = include_bytes!("../../res/cobble-diffuse.png");
            let normal_bytes = include_bytes!("../../res/cobble-normal.png");

            let diffuse_texture = texture::Texture::from_bytes(
                &device,
                &queue,
                diffuse_bytes,
                "res/alt-diffuse.png",
                false,
            )
            .unwrap();
            let normal_texture = texture::Texture::from_bytes(
                &device,
                &queue,
                normal_bytes,
                "res/alt-normal.png",
                false,
            )
            .unwrap();

            obj_model::ObjMaterial::new(
                &device,
                "alt-material",
                diffuse_texture,
                normal_texture,
                &texture_bind_group_layout,
            )
        };

        let obj_model_transforms = (0..10)
            .flat_map(|z| {
                (0..10).map(move |x| {
                    let mut transform = Transform::new();
                    transform
                        .set_position(cgmath::Point3::new(
                            x as f32 * 4.0 - 18.0,
                            -4.0,
                            z as f32 * 4.0 - 18.0,
                        ))
                        .set_rotation_x(x as f32)
                        .set_rotation_z(z as f32);

                    transform
                })
            })
            .collect::<Vec<_>>();

        let obj_model_transform_data = obj_model_transforms
            .iter()
            .map(TransformRaw::from)
            .collect::<Vec<_>>();

        let obj_model_transform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Obj Model Transform Buffer"),
                contents: bytemuck::cast_slice(&obj_model_transform_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let pmx_model_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("PMX Model Texture Bind Group Layout"),
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
                    },
                ],
            });

        let pmx_lumine_model = match resources::load_pmx_model(
            "lumine",
            "lumine.pmx",
            &device,
            &queue,
            &pmx_model_texture_bind_group_layout,
        ) {
            Ok(model) => model,
            Err(e) => {
                error!("Failed to load PMX model: {:?}", e);
                std::process::exit(1);
            }
        };
        let mut pmx_lumine_model_transform = Transform::new();
        pmx_lumine_model_transform
            .set_position_x(-20.0)
            .set_position_z(40.0)
            .set_rotation_y(std::f32::consts::PI);
        let pmx_lumine_model_transform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("PMX Lumine Model Transform Buffer"),
                contents: bytemuck::cast_slice(&[TransformRaw::from(&pmx_lumine_model_transform)]),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let pmx_barbara_model = match resources::load_pmx_model(
            "barbara",
            "barbara.pmx",
            &device,
            &queue,
            &pmx_model_texture_bind_group_layout,
        ) {
            Ok(model) => model,
            Err(e) => {
                error!("Failed to load PMX model: {:?}", e);
                std::process::exit(1);
            }
        };
        let mut pmx_barbara_model_transform = Transform::new();
        pmx_barbara_model_transform
            .set_position_x(20.0)
            .set_position_z(40.0)
            .set_rotation_y(std::f32::consts::PI);
        let pmx_barbara_model_transform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("PMX Barbara Model Transform Buffer"),
                contents: bytemuck::cast_slice(&[TransformRaw::from(&pmx_barbara_model_transform)]),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let mut plane = Square::new();
        plane
            .transform
            .set_position_y(30.0)
            .set_position_z(10.0)
            .set_scale(50.0);

        let transform_data = vec![TransformRaw::from(&plane.transform)];
        let plane_transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Plane Transform Buffer"),
            contents: bytemuck::cast_slice(&transform_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        shape_factory.create_geometry(&device, ShapeType::Square);

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &surface_config, "depth_texture");
        let depth_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    },
                ],
            });

        let depth_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                },
            ],
        });

        let sphere_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../res/shader/sphere.wgsl"));
        let sphere_render_pipeline = Self::create_render_pipeline(
            "sphere",
            &device,
            &[&camera_bind_group_layout, &light_bind_group_layout],
            Self::create_vertex_state(
                &sphere_shader,
                &[vertex::ModelVertex::desc(), TransformRaw::desc()],
            ),
            Self::create_fragment_state(
                &sphere_shader,
                &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            ),
            Self::create_primitive_state(),
            Self::create_depth_stencil_state(texture::Texture::DEPTH_FORMAT),
        );

        let obj_model_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../res/shader/obj_model.wgsl"));
        let obj_model_render_pipeline = Self::create_render_pipeline(
            "obj_model",
            &device,
            &[
                &texture_bind_group_layout,
                &camera_bind_group_layout,
                &light_bind_group_layout,
            ],
            Self::create_vertex_state(
                &obj_model_shader,
                &[vertex::ModelVertex::desc(), TransformRaw::desc()],
            ),
            Self::create_fragment_state(
                &obj_model_shader,
                &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            ),
            Self::create_primitive_state(),
            Self::create_depth_stencil_state(texture::Texture::DEPTH_FORMAT),
        );

        let lumine_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../res/shader/pmx_model.wgsl"));
        let pmx_lumine_model_render_pipeline = Self::create_render_pipeline(
            "lumine",
            &device,
            &[
                &pmx_model_texture_bind_group_layout,
                &camera_bind_group_layout,
                &light_bind_group_layout,
            ],
            Self::create_vertex_state(
                &lumine_shader,
                &[vertex::ModelVertex::desc(), TransformRaw::desc()],
            ),
            Self::create_fragment_state(
                &lumine_shader,
                &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            ),
            Self::create_primitive_state(),
            Self::create_depth_stencil_state(texture::Texture::DEPTH_FORMAT),
        );

        let barbara_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../res/shader/pmx_model.wgsl"));
        let pmx_barbara_model_render_pipeline = Self::create_render_pipeline(
            "barbara",
            &device,
            &[
                &pmx_model_texture_bind_group_layout,
                &camera_bind_group_layout,
                &light_bind_group_layout,
            ],
            Self::create_vertex_state(
                &barbara_shader,
                &[vertex::ModelVertex::desc(), TransformRaw::desc()],
            ),
            Self::create_fragment_state(
                &barbara_shader,
                &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            ),
            Self::create_primitive_state(),
            Self::create_depth_stencil_state(texture::Texture::DEPTH_FORMAT),
        );

        let light_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../res/shader/light.wgsl"));
        let light_render_pipeline = Self::create_render_pipeline(
            "light",
            &device,
            &[&camera_bind_group_layout, &light_bind_group_layout],
            Self::create_vertex_state(&light_shader, &[vertex::ModelVertex::desc()]),
            Self::create_fragment_state(
                &light_shader,
                &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            ),
            Self::create_primitive_state(),
            Self::create_depth_stencil_state(texture::Texture::DEPTH_FORMAT),
        );

        let depth_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../res/shader/depth.wgsl"));
        let depth_render_pipeline = Self::create_render_pipeline(
            "depth",
            &device,
            &[&camera_bind_group_layout, &depth_bind_group_layout],
            Self::create_vertex_state(
                &depth_shader,
                &[vertex::ModelVertex::desc(), TransformRaw::desc()],
            ),
            Self::create_fragment_state(
                &depth_shader,
                &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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

            light_uniform,
            light_buffer,
            light_bind_group,
            light_render_pipeline,

            shape_factory,
            spheres,
            sphere_transform_buffer,
            sphere_render_pipeline,

            obj_model,
            debug_material,
            obj_model_transforms,
            obj_model_transform_buffer,
            obj_model_render_pipeline,

            pmx_lumine_model,
            pmx_lumine_model_transform,
            pmx_lumine_model_transform_buffer,
            pmx_lumine_model_render_pipeline,

            pmx_barbara_model,
            pmx_barbara_model_transform,
            pmx_barbara_model_transform_buffer,
            pmx_barbara_model_render_pipeline,

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
        self.depth_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                },
            ],
        });

        self.camera.aspect = self.surface_config.width as f32 / self.surface_config.height as f32;
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // 普通のRenderPass
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

            // ライト
            render_pass.set_pipeline(&self.light_render_pipeline);
            render_pass.draw_light_model(
                &self.obj_model,
                &self.camera_bind_group,
                &self.light_bind_group,
            );

            // 球
            let sphere_geometry_buffers = self.shape_factory.get_geometry(&SPHERE).unwrap();
            render_pass.set_pipeline(&self.sphere_render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.light_bind_group, &[]);
            render_pass.set_vertex_buffer(0, sphere_geometry_buffers.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.sphere_transform_buffer.slice(..));
            render_pass.set_index_buffer(
                sphere_geometry_buffers.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.draw_indexed(
                0..sphere_geometry_buffers.indices_count,
                0,
                0..self.spheres.len() as _,
            );

            // Obj Model Rendering
            render_pass.set_pipeline(&self.obj_model_render_pipeline);
            render_pass.set_vertex_buffer(1, self.obj_model_transform_buffer.slice(..));
            render_pass.draw_model_instanced_with_material(
                &self.obj_model,
                &self.debug_material,
                0..self.obj_model_transforms.len() as u32,
                &self.camera_bind_group,
                &self.light_bind_group,
            );

            // PMX Model Rendering
            // Lumine
            render_pass.set_pipeline(&self.pmx_lumine_model_render_pipeline);
            render_pass.set_vertex_buffer(0, self.pmx_lumine_model.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.pmx_lumine_model_transform_buffer.slice(..));
            for parts in &self.pmx_lumine_model.parts {
                render_pass.set_bind_group(0, &parts.texture_bind_group, &[]);
                render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                render_pass.set_bind_group(2, &self.light_bind_group, &[]);
                render_pass
                    .set_index_buffer(parts.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..parts.num_elements, 0, 0..1);
            }

            // Barbara
            render_pass.set_pipeline(&self.pmx_barbara_model_render_pipeline);
            render_pass.set_vertex_buffer(0, self.pmx_barbara_model.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.pmx_barbara_model_transform_buffer.slice(..));
            for parts in &self.pmx_barbara_model.parts {
                render_pass.set_bind_group(0, &parts.texture_bind_group, &[]);
                render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                render_pass.set_bind_group(2, &self.light_bind_group, &[]);
                render_pass
                    .set_index_buffer(parts.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..parts.num_elements, 0, 0..1);
            }
        }

        // 深度テクスチャRenderPass
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

            let plane_geometry_buffers =
                self.shape_factory.get_geometry(&ShapeType::Square).unwrap();

            render_pass.set_vertex_buffer(0, plane_geometry_buffers.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.plane_transform_buffer.slice(..));
            render_pass.set_index_buffer(
                plane_geometry_buffers.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.draw_indexed(0..plane_geometry_buffers.indices_count, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn key_input(&mut self, key_event: &KeyEvent) {
        self.camera_controller.process_key_events(key_event);
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);

        let old_position: cgmath::Vector3<f32> = self.light_uniform.position.into();
        self.light_uniform.position =
            (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(0.1))
                * old_position)
                .into();

        let time = self.instant_time.elapsed().as_secs_f32();

        for (index, sphere) in &mut self.spheres.iter_mut().enumerate() {
            sphere
                .transform
                .set_position(cgmath::Point3::new(
                    (index % 10) as f32 * 3.0 - 13.5,
                    (time * 0.5 + index as f32).sin() * 10.0,
                    (index / 10) as f32 * 3.0 - 13.5,
                ))
                .set_rotation(cgmath::Euler::new(
                    cgmath::Rad(time * 4.0),
                    cgmath::Rad(time * 1.4),
                    cgmath::Rad(time * 2.6),
                ));
        }

        let transform_data = self
            .spheres
            .iter()
            .map(|x| &x.transform)
            .map(TransformRaw::from)
            .collect::<Vec<_>>();

        self.sphere_transform_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Sphere Transform Buffer"),
                    contents: bytemuck::cast_slice(&transform_data),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        self.queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light_uniform]),
        );
    }
}

/// private impl
impl Renderer<'_> {
    fn create_adapter(instance: &wgpu::Instance, surface: &wgpu::Surface) -> wgpu::Adapter {
        let request = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        });

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
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&layout_label),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        let label = format!("{}_render_pipeline", label);
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
        })
    }

    fn create_vertex_state<'a>(
        shader: &'a wgpu::ShaderModule,
        buffers: &'a [wgpu::VertexBufferLayout],
    ) -> wgpu::VertexState<'a> {
        wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers,
            compilation_options: Default::default(),
        }
    }

    fn create_fragment_state<'a>(
        shader: &'a wgpu::ShaderModule,
        targets: &'a [Option<wgpu::ColorTargetState>],
    ) -> Option<wgpu::FragmentState<'a>> {
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

    fn create_depth_stencil_state(format: wgpu::TextureFormat) -> Option<wgpu::DepthStencilState> {
        Some(wgpu::DepthStencilState {
            format: format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        })
    }
}
