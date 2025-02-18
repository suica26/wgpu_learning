use cgmath::{EuclideanSpace, Euler, Matrix3, Matrix4, Point3, Quaternion, Rad, Vector3};

/// Transform情報を保持する構造体
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

    pub fn add_position(&mut self, position: Vector3<f32>) -> &mut Self {
        self.position += position;
        self
    }

    pub fn add_position_x(&mut self, x: f32) -> &mut Self {
        self.position.x += x;
        self
    }

    pub fn add_position_y(&mut self, y: f32) -> &mut Self {
        self.position.y += y;
        self
    }

    pub fn add_position_z(&mut self, z: f32) -> &mut Self {
        self.position.z += z;
        self
    }

    pub fn add_rotation(&mut self, rotation: Euler<Rad<f32>>) -> &mut Self {
        self.add_rotation_x(rotation.x.0);
        self.add_rotation_y(rotation.y.0);
        self.add_rotation_z(rotation.z.0);
        self
    }

    pub fn add_rotation_x(&mut self, x: f32) -> &mut Self {
        self.angle.x += Rad(x);
        self
    }

    pub fn add_rotation_y(&mut self, y: f32) -> &mut Self {
        self.angle.y += Rad(y);
        self
    }

    pub fn add_rotation_z(&mut self, z: f32) -> &mut Self {
        self.angle.z += Rad(z);
        self
    }

    pub fn add_scale(&mut self, scale: f32) -> &mut Self {
        self.scale += Vector3::new(scale, scale, scale);
        self
    }

    pub fn add_scale_by_vector(&mut self, scale: Vector3<f32>) -> &mut Self {
        self.scale += scale;
        self
    }

    pub fn add_scale_x(&mut self, x: f32) -> &mut Self {
        self.scale.x += x;
        self
    }

    pub fn add_scale_y(&mut self, y: f32) -> &mut Self {
        self.scale.y += y;
        self
    }

    pub fn add_scale_z(&mut self, z: f32) -> &mut Self {
        self.scale.z += z;
        self
    }

    pub fn set_position(&mut self, position: Point3<f32>) -> &mut Self {
        self.position = position;
        self
    }

    pub fn set_position_x(&mut self, x: f32) -> &mut Self {
        self.position.x = x;
        self
    }

    pub fn set_position_y(&mut self, y: f32) -> &mut Self {
        self.position.y = y;
        self
    }

    pub fn set_position_z(&mut self, z: f32) -> &mut Self {
        self.position.z = z;
        self
    }

    pub fn set_rotation(&mut self, rotation: Euler<Rad<f32>>) -> &mut Self {
        self.angle = rotation;
        self
    }

    pub fn set_rotation_x(&mut self, x: f32) -> &mut Self {
        self.angle.x = Rad(x);
        self
    }

    pub fn set_rotation_y(&mut self, y: f32) -> &mut Self {
        self.angle.y = Rad(y);
        self
    }

    pub fn set_rotation_z(&mut self, z: f32) -> &mut Self {
        self.angle.z = Rad(z);
        self
    }

    pub fn set_scale(&mut self, scale: f32) -> &mut Self {
        self.scale = Vector3::new(scale, scale, scale);
        self
    }

    pub fn set_scale_by_vector(&mut self, scale: Vector3<f32>) -> &mut Self {
        self.scale = scale;
        self
    }

    pub fn set_scale_x(&mut self, x: f32) -> &mut Self {
        self.scale.x = x;
        self
    }

    pub fn set_scale_y(&mut self, y: f32) -> &mut Self {
        self.scale.y = y;
        self
    }

    pub fn set_scale_z(&mut self, z: f32) -> &mut Self {
        self.scale.z = z;
        self
    }
}

/// Transform情報をGPUに渡すための構造体
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformRaw {
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3],
}

impl TransformRaw {
    pub fn from(transform: &Transform) -> Self {
        Self {
            model: transform.get_matrix().into(),
            normal: Matrix3::from(Quaternion::from(transform.get_rotation())).into(),
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<TransformRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 5,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 6,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 7,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 8,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x3,
                    shader_location: 9,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x3,
                    shader_location: 10,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x3,
                    shader_location: 11,
                },
            ],
        }
    }
}
