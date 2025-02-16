use log::{debug, error};

use crate::system::shape_geometry::ShapeGeometry;
use crate::system::transform::Transform;
use crate::system::vertex::Vertex;

/// プリミティブ形状の種類
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ShapeType {
    /// 立方体
    Cube,
    /// 球 (分割数)
    Sphere(u16),
    /// 円柱 (分割数)
    Cylinder(u16),
    /// 円錐 (分割数)
    Cone(u16),
    /// トーラス (分割数)
    Torus(u16),
    /// 正方形
    Square,
    /// その他(ラベル)
    Other(String),
}

/// 正方形
pub struct Square {
    pub transform: Transform,
}

impl Square {
    pub fn new() -> Self {
        Self { transform: Transform::new() }
    }
}

/// 球
pub struct Sphere {
    pub transform: Transform,
}

impl Sphere {
    pub fn new(radius: f32) -> Self {
        if radius <= 0.0 {
            error!("can't create sphere by radius less than or equal to 0.");
        }

        let mut transform = Transform::new();
        transform.set_scale(cgmath::Vector3::new(radius, radius, radius));

        Self { transform }
    }

    pub fn radius(&self) -> f32 {
        self.transform.get_scale().x
    }

    pub fn set_radius(&mut self, radius: f32) {
        if radius <= 0.0 {
            error!("can't set radius less than or equal to 0.");
        }

        self.transform.set_scale(cgmath::Vector3::new(radius, radius, radius));
    }
}