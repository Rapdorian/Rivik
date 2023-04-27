/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! Render pipeline for a basic 3d mesh

use once_cell::sync::Lazy;
use wgpu::RenderPipeline;

use crate::{shader, transform};

use super::{simple, GBuffer};

#[allow(missing_docs)]
mod vertex {
    use bytemuck::{Pod, Zeroable};
    use wgpu_macros::VertexLayout;

    #[repr(C)]
    #[derive(Debug, Clone, Copy, Zeroable, Pod, VertexLayout, Default)]
    /// A basic 3d vertex
    pub struct MeshVertex {
        pub pos: [f32; 3],
        pub norm: [f32; 3],
        pub uv: [f32; 2],
    }
}

pub use vertex::MeshVertex;

/// Render pipeline for rendering a simple Phong shaded mesh
pub static PIPELINE: Lazy<RenderPipeline> = Lazy::new(|| {
    GBuffer::geom_pipeline(
        &shader!("../shaders/mesh.wgsl").unwrap(),
        &[&*simple::TEX_LAYOUT, transform::layout()],
        MeshVertex::LAYOUT,
    )
});
