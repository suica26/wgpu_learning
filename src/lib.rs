use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

pub struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(event_loop.create_window(Window::default_attributes()).unwrap());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.close_requested(event_loop),
            WindowEvent::RedrawRequested => self.window.as_ref().unwrap().request_redraw(),
            _ => {}
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None
        }
    }

    fn close_requested(&self, event_loop: &ActiveEventLoop) {
        println!("The close button was pressed; stopping");
        event_loop.exit();
    }
}