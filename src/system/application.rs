use std::process;
use std::sync::Arc;

use log::{debug, error, info};
use wgpu::{RenderPassDescriptor, SurfaceError};
use winit::application::ApplicationHandler;
use winit::event::{DeviceId, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::Key::Named;
use winit::keyboard::NamedKey::Escape;
use winit::window::{Fullscreen, Window, WindowButtons, WindowId};

use crate::system::renderer::Renderer;

pub struct Application<'a> {
    pub window: Option<Arc<Window>>,
    pub renderer: Option<Renderer<'a>>,
}

impl<'a> ApplicationHandler for Application<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("wgpu learning");
        let window = match event_loop.create_window(window_attributes) {
            Ok(n) => Arc::new(n),
            Err(e) => {
                error!("Failed to create window: {:?}", e);
                process::exit(1);
            }
        };

        window.set_enabled_buttons(WindowButtons::CLOSE | WindowButtons::MINIMIZE);
        info!("Window created");

        let renderer = Renderer::new(window.clone());

        info!("initialization done");
        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.close_requested(event_loop),
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => self.keyboard_input(device_id, event, is_synthetic),
            WindowEvent::RedrawRequested => self.redraw_requested(),
            WindowEvent::Resized(size) => self.renderer.as_mut().unwrap().resize(size.width, size.height),
            _ => {}
        }
    }
}

impl<'a> Application<'a> {
    fn close_requested(&self, event_loop: &ActiveEventLoop) {
        println!("The close button was pressed; stopping");
        event_loop.exit();
    }

    fn keyboard_input(&self, device_id: DeviceId, event: KeyEvent, is_synthetic: bool) {
        debug!("Keyboard input: {:?}", event);
        if event.logical_key == Named(Escape) {
            println!("The escape key was pressed; stopping");
            process::exit(0);
        }
    }

    fn redraw_requested(&mut self) {
        self.window.as_ref().unwrap().request_redraw();

        if self.renderer.is_none() {
            return;
        }

        match self.renderer.as_mut().unwrap().render() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.renderer.as_mut().unwrap().resize(0, 0)
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                error!("Out of memory");
                process::exit(1);
            }
            Err(e) => {
                error!("Failed to render: {:?}", e);
                process::exit(1);
            }
        }
    }

    fn update(&mut self) {}
}
