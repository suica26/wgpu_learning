use cgmath::{Quaternion, Vector3};

use crate::system::transform::Transform;

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

    pub fn get_normal(&self) -> Vector3<f32> {
        Quaternion::from(self.transform.get_rotation()) * Vector3::unit_z()
    }
}

/// 球
pub struct Sphere {
    pub transform: Transform,
}

impl Sphere {
    pub fn new() -> Self {
        Self { transform: Transform::new() }
    }

    pub fn radius(&self) -> f32 {
        self.transform.get_scale().x
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.transform.set_scale_by_vector(Vector3::new(radius, radius, radius));
    }
}