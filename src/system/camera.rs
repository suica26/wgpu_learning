use cgmath::{EuclideanSpace, Matrix, Point3, Rad, SquareMatrix};

use super::transform::Transform;

pub struct Camera {
    pub transform: Transform,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            transform: Transform::new(),
        }
    }

    fn calc_matrix(&self) -> cgmath::Matrix4<f32> {
        let eye = self.transform.get_position();
        let forward = self.transform.get_forward();
        let up = self.transform.get_up();

        let target = Point3::from_vec(eye.to_vec() + forward);

        cgmath::Matrix4::look_at_rh(eye, target, up)
    }
}

pub struct Projection {
    pub aspect: f32,
    pub fovy: Rad<f32>,
    pub znear: f32,
    pub zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, heigh: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / heigh as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, heigh: u32) {
        self.aspect = width as f32 / heigh as f32;
    }

    pub fn calc_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view: [[f32; 4]; 4],
    view_proj: [[f32; 4]; 4],
    inv_proj: [[f32; 4]; 4],
    inv_view: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_position: [0.0; 4],
            view: cgmath::Matrix4::identity().into(),
            view_proj: cgmath::Matrix4::identity().into(),
            inv_proj: cgmath::Matrix4::identity().into(),
            inv_view: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_position = camera.transform.get_position().to_homogeneous().into();
        let proj = projection.calc_matrix();
        let view = camera.calc_matrix();
        self.view = view.into();
        self.view_proj = (proj * view).into();
        self.inv_proj = proj.invert().unwrap().into();
        self.inv_view = view.transpose().into();
    }
}
