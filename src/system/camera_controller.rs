use cgmath::Vector3;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard;

use crate::system::camera::Camera;

pub struct CameraController {
    pub speed: f32,
    pub rotation_speed: f32,
    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
    pub is_up_pressed: bool,
    pub is_down_pressed: bool,
    pub is_right_rotate_pressed: bool,
    pub is_left_rotate_pressed: bool,
    pub is_up_rotate_pressed: bool,
    pub is_down_rotate_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            rotation_speed: 0.01,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            is_right_rotate_pressed: false,
            is_left_rotate_pressed: false,
            is_up_rotate_pressed: false,
            is_down_rotate_pressed: false,
        }
    }

    pub fn process_key_events(&mut self, key_event: &KeyEvent) {
        use keyboard::{KeyCode, PhysicalKey};

        let is_pressed = key_event.state == ElementState::Pressed;

        match key_event.physical_key {
            PhysicalKey::Code(KeyCode::KeyW) => {
                self.is_forward_pressed = is_pressed;
            }
            PhysicalKey::Code(KeyCode::KeyS) => {
                self.is_backward_pressed = is_pressed;
            }
            PhysicalKey::Code(KeyCode::KeyA) => {
                self.is_left_pressed = is_pressed;
            }
            PhysicalKey::Code(KeyCode::KeyD) => {
                self.is_right_pressed = is_pressed;
            }
            PhysicalKey::Code(KeyCode::Space) => {
                self.is_up_pressed = is_pressed;
            }
            PhysicalKey::Code(KeyCode::ShiftLeft) => {
                self.is_down_pressed = is_pressed;
            }
            PhysicalKey::Code(KeyCode::KeyE) => {
                self.is_right_rotate_pressed = is_pressed;
            }
            PhysicalKey::Code(KeyCode::KeyQ) => {
                self.is_left_rotate_pressed = is_pressed;
            }
            PhysicalKey::Code(KeyCode::KeyR) => {
                self.is_up_rotate_pressed = is_pressed;
            }
            PhysicalKey::Code(KeyCode::KeyF) => {
                self.is_down_rotate_pressed = is_pressed;
            }
            _ => (),
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        let up = Vector3::unit_y();
        let left = camera.transform.get_left();
        let forward = left.cross(up);
        if self.is_forward_pressed {
            camera.transform.add_position(forward * self.speed);
        }
        if self.is_backward_pressed {
            camera.transform.add_position(-forward * self.speed);
        }

        if self.is_left_pressed {
            camera.transform.add_position(left * self.speed);
        }
        if self.is_right_pressed {
            camera.transform.add_position(-left * self.speed);
        }

        if self.is_up_pressed {
            camera.transform.add_position(up * self.speed);
        }
        if self.is_down_pressed {
            camera.transform.add_position(-up * self.speed);
        }

        if self.is_left_rotate_pressed {
            camera.transform.add_rotation_y(self.rotation_speed);
        }
        if self.is_right_rotate_pressed {
            camera.transform.add_rotation_y(-self.rotation_speed);
        }

        const MAX_PITCH: f32 = 80.0 * std::f32::consts::PI / 180.0;
        if self.is_up_rotate_pressed {
            let current_up = camera.transform.get_rotation().x;
            let new_up = (current_up.0 - self.rotation_speed).max(-MAX_PITCH);
            camera.transform.set_rotation_x(new_up);
        }
        if self.is_down_rotate_pressed {
            let current_up = camera.transform.get_rotation().x;
            let new_up = (current_up.0 + self.rotation_speed).min(MAX_PITCH);
            camera.transform.set_rotation_x(new_up);
        }
    }
}
