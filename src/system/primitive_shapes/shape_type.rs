/// プリミティブ形状の種類
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
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
    /// 平面
    Rect,
    /// 四角形
    Quad,
}