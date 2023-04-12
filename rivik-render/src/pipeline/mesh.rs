use bytemuck::{Pod, Zeroable};
use once_cell::sync::Lazy;
use wgpu::RenderPipeline;
use wgpu_macros::VertexLayout;

use crate::{shader, transform};

use super::{simple, GBuffer};

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod, VertexLayout, Default)]
pub struct MeshVertex {
    pub pos: [f32; 3],
    pub norm: [f32; 3],
    pub uv: [f32; 2],
}

/// Render pipeline for rendering a simple Phong shaded mesh
pub static PIPELINE: Lazy<RenderPipeline> = Lazy::new(|| {
    GBuffer::geom_pipeline(
        &shader!("../shaders/mesh.wgsl").unwrap(),
        &[&*simple::TEX_LAYOUT, transform::layout()],
        MeshVertex::LAYOUT,
    )
});
