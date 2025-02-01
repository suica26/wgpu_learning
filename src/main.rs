use crate::system::application::Application;

mod system;

fn main() {
    let mut app = Application::new();
    app.run();
}