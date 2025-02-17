use crate::system::shape_geometry::ShapeGeometry;
use crate::system::vertex::ModelVertex;

pub fn create_square_geometry() -> ShapeGeometry {
    let vertices = vec![
        ModelVertex {
            position: [-0.5, 0.5, 0.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 1.0],
        },
        ModelVertex {
            position: [0.5, 0.5, 0.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [1.0, 1.0],
        },
        ModelVertex {
            position: [0.5, -0.5, 0.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [1.0, 0.0],
        },
        ModelVertex {
            position: [-0.5, -0.5, 0.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 0.0],
        },
    ];

    let indices = vec![0, 1, 2, 0, 2, 3];

    ShapeGeometry::from(vertices.to_vec(), indices.to_vec())
}