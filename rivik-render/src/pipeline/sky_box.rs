/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

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
