//! Render pipeline for a skybox

use once_cell::sync::Lazy;
use wgpu::RenderPipeline;

use crate::{shader, transform};

use super::{mesh::MeshVertex, simple, GBuffer};

/// Render pipeline for a skybox
pub static PIPELINE: Lazy<RenderPipeline> = Lazy::new(|| {
    GBuffer::geom_no_depth_pipeline(
        &shader!("../shaders/skybox.wgsl").unwrap(),
        &[&*simple::TEX_LAYOUT, transform::layout()],
        MeshVertex::LAYOUT,
    )
});
