use crate::system::vertex::Vertex;

/// 3D形状を表す構造体
pub struct ShapeGeometry {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub indices_count: usize,
}

impl ShapeGeometry {
    pub fn from(vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
        let indices_count = indices.len();
        Self {
            vertices,
            indices,
            indices_count,
        }
    }
}