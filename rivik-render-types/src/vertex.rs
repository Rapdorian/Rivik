use bytemuck::{Pod, Zeroable};

/// Vertex type
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub norm: [f32; 3],
    pub uv: [f32; 2],
}
