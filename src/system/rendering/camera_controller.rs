use cgmath::InnerSpace;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard;

use crate::system::rendering::camera::Camera;

pub struct CameraController {
    pub speed: f32,
    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_events(&mut self, key_event: &KeyEvent) {
        use keyboard::{KeyCode, PhysicalKey};

        let is_pressed = key_event.state == ElementState::Pressed;

        match key_event.physical_key {
            PhysicalKey::Code(KeyCode::KeyW) | PhysicalKey::Code(KeyCode::ArrowUp) => {
                self.is_forward_pressed = is_pressed;
            }
            PhysicalKey::Code(KeyCode::KeyS) | PhysicalKey::Code(KeyCode::ArrowDown) => {
                self.is_backward_pressed = is_pressed;
            }
            PhysicalKey::Code(KeyCode::KeyA) | PhysicalKey::Code(KeyCode::ArrowLeft) => {
                self.is_left_pressed = is_pressed;
            }
            PhysicalKey::Code(KeyCode::KeyD) | PhysicalKey::Code(KeyCode::ArrowRight) => {
                self.is_right_pressed = is_pressed;
            }
            _ => (),
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;

        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}