use crate::system::shapes::ShapeType;
use crate::system::transform::Transform;

pub struct Model<'a> {
    pub transform: Transform,
    shape_type: ShapeType,
    vs_module: &'a wgpu::ShaderModule,
    fs_module: &'a wgpu::ShaderModule,
}