use std::collections::HashMap;

use crate::system::primitive_shapes::shape_type::ShapeType;
use crate::system::rendering::shape_geometry::ShapeGeometry;

pub struct ShapeGeometryCache {
    cache: HashMap<ShapeType, ShapeGeometry>,
}