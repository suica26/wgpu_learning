use std::collections::HashMap;

use wgpu::util::DeviceExt;

use crate::system::shapes::ShapeType;
use crate::system::vertex::ModelVertex;

mod sphere;
mod square;

/// 3D形状を表す構造体
pub struct ShapeGeometry {
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u16>,
    pub indices_count: usize,
}

impl ShapeGeometry {
    pub fn from(vertices: Vec<ModelVertex>, indices: Vec<u16>) -> Self {
        let indices_count = indices.len();
        Self {
            vertices,
            indices,
            indices_count,
        }
    }
}

/// 3D形状のバッファを表す構造体
pub struct ShapeGeometryBuffers {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub indices_count: u32,
}

impl From<(&wgpu::Device, ShapeGeometry)> for ShapeGeometryBuffers {
    fn from(value: (&wgpu::Device, ShapeGeometry)) -> Self {
        let (device, geometry) = value;

        Self {
            vertex_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&geometry.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            index_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&geometry.indices),
                usage: wgpu::BufferUsages::INDEX,
            }),
            indices_count: geometry.indices_count as u32,
        }
    }
}

/// 3D形状のバッファを生成するファクトリ
pub struct ShapeGeometryFactory {
    geometries: HashMap<ShapeType, ShapeGeometryBuffers>,
}

impl ShapeGeometryFactory {
    pub fn new() -> Self {
        Self { geometries: HashMap::new() }
    }

    pub fn create_geometry(&mut self, device: &wgpu::Device, shape_type: ShapeType) -> &ShapeGeometryBuffers {
        self.geometries.entry(shape_type)
            .or_insert_with_key(|st| {
                let geometry = match st {
                    ShapeType::Sphere(div) => sphere::create_sphere_geometry(*div),
                    ShapeType::Square => square::create_square_geometry(),
                    _ => panic!("Unsupported shape type: {:?}", st),
                };

                ShapeGeometryBuffers::from((device, geometry))
            })
    }

    pub fn get_geometry(&self, shape_type: &ShapeType) -> Option<&ShapeGeometryBuffers> {
        self.geometries.get(shape_type)
    }
}