use std::sync::Arc;

use log::{error, info};
use pollster::block_on;
use wgpu::{Device, DeviceDescriptor, Instance, InstanceDescriptor, Queue, RequestAdapterOptions, Surface, SurfaceCapabilities, SurfaceConfiguration};
use winit::dpi::PhysicalSize;
use winit::window::Window;

/// 描画を行う構造体
pub struct Renderer<'a> {
    pub surface: Surface<'a>,
    pub device: Device,
    pub queue: Queue,
    pub surface_caps: SurfaceCapabilities,
    pub surface_format: wgpu::TextureFormat,
}

impl<'a> Renderer<'a> {
    pub fn new(window: Arc<Window>) -> Self {
        let instance = Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            ..Default::default()
        });

        let surface = match instance.create_surface(Arc::clone(&window)) {
            Ok(surface) => surface,
            Err(e) => {
                error!("Failed to create surface: {:?}", e);
                std::process::exit(1);
            }
        };

        let request = instance.request_adapter(&RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        });

        let adapter = match block_on(request) {
            Some(adapter) => {
                info!("Adapter: {:?}", adapter.get_info());
                adapter
            }
            None => {
                error!("Failed to find a suitable adapter");
                std::process::exit(1);
            }
        };

        let request = adapter.request_device(
            &DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        );

        let (device, queue) = match block_on(request) {
            Ok(n) => n,
            Err(e) => {
                error!("Failed to create device and queue: {:?}", e);
                std::process::exit(1);
            }
        };

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
            &SurfaceConfiguration {
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

        Self {
            surface,
            device,
            queue,
            surface_caps,
            surface_format,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface.configure(
            &self.device,
            &SurfaceConfiguration {
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
}