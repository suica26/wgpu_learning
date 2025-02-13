use crate::system::game_object::transform::Transform;
use crate::system::rendering::shape_geometry::ShapeGeometry;
use crate::system::rendering::vertex::Vertex;

pub struct Plane {
    pub transform: Transform,
}

impl Plane {
    pub fn new() -> Self {
        Self { transform: Transform::new() }
    }

    pub fn create_geometry() -> ShapeGeometry {
        let vertices = vec![
            Vertex {
                position: [-0.5, 0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                tex_coords: [0.0, 0.0],
            },
        ];

        let indices = vec![0, 1, 2, 0, 2, 3];

        ShapeGeometry::from(vertices.to_vec(), indices.to_vec())
    }
}