use winit::event_loop::{ControlFlow, EventLoop};

use wgpu_learning::App;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();
    match event_loop.run_app(&mut app) {
        Ok(_) => println!("The event loop has exited cleanly"),
        Err(e) => eprintln!("An error occurred: {:?}", e),
    }
}
