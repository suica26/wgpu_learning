use std::env;

use log::{debug, error};
use winit::event_loop::{ControlFlow, EventLoop};

use crate::system::application::Application;

mod system;
mod primitive_shapes;

fn main() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = Application {
        window: None,
        renderer: None,
    };
    match event_loop.run_app(&mut app) {
        Ok(_) => debug!("The event loop has exited cleanly"),
        Err(e) => error!("An error occurred: {:?}", e),
    }
}
