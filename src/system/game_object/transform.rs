use cgmath::{EuclideanSpace, Euler, Matrix4, Point3, Quaternion, Rad, SquareMatrix, Vector3};
use winit::dpi::Position;

/// Transform component
pub struct Transform {
    position: Point3<f32>,
    angle: Euler<Rad<f32>>,
    scale: Vector3<f32>,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Point3::new(0.0, 0.0, 0.0),
            angle: Euler::new(Rad(0.0), Rad(0.0), Rad(0.0)),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn get_position(&self) -> Point3<f32> {
        self.position
    }

    pub fn get_rotation(&self) -> Euler<Rad<f32>> {
        self.angle
    }

    pub fn get_scale(&self) -> Vector3<f32> {
        self.scale
    }

    pub fn get_matrix(&self) -> Matrix4<f32> {
        let translation = Matrix4::from_translation(self.position.to_vec());
        let rotation = Matrix4::from(Quaternion::from(self.angle));
        let scale = Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);

        translation * rotation * scale
    }

    pub fn set_position(&mut self, position: Point3<f32>) -> &mut Self {
        self.position = position;
        self
    }

    pub fn set_rotation(&mut self, rotation: Euler<Rad<f32>>) -> &mut Self {
        self.angle = rotation;
        self
    }

    pub fn set_scale(&mut self, scale: Vector3<f32>) -> &mut Self {
        self.scale = scale;
        self
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformUniform {
    pub model_matrix: [[f32; 4]; 4],
}

impl TransformUniform {
    pub fn new() -> Self {
        Self { model_matrix: Matrix4::identity().into() }
    }

    pub fn update(&mut self, transform: &Transform) {
        self.model_matrix = transform.get_matrix().into();
    }
}