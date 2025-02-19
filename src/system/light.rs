#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    /// ユニフォームバッファは16バイトアライメントが必要
    pub _padding: f32,
    pub color: [f32; 3],
    /// ユニフォームバッファは16バイトアライメントが必要
    pub _padding2: f32,
}
