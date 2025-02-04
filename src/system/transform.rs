use cgmath::{Matrix4, Point3, SquareMatrix};

/// Transform component
pub struct Transform {
    matrix: Matrix4<f32>,
}

impl Transform {
    pub fn new() -> Self {
        Self { matrix: Matrix4::identity() }
    }

    pub fn matrix(&self) -> &Matrix4<f32> {
        &self.matrix
    }

    pub fn set_position(&mut self, position: Point3<f32>) {
        self.matrix.w.x = position.x;
        self.matrix.w.y = position.y;
        self.matrix.w.z = position.z;
    }

    pub fn get_position(&self) -> Point3<f32> {
        Point3::new(self.matrix.w.x, self.matrix.w.y, self.matrix.w.z)
    }

    pub fn set_rotation(&mut self, rotation: Matrix4<f32>) {
        self.matrix = rotation * self.matrix;
    }

    pub fn set_scale(&mut self, scale: Point3<f32>) {
        self.matrix.x.x = scale.x;
        self.matrix.y.y = scale.y;
        self.matrix.z.z = scale.z;
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
        self.model_matrix = transform.matrix.into();
    }
}