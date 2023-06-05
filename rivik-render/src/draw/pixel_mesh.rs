/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! Utilities for rendering a pixelated mesh
use std::{rc::Rc, sync::Arc};

use assets::formats::mesh::{Mesh, Vert};
use wgpu::{
    RenderBundle, RenderBundleDescriptor, RenderBundleEncoderDescriptor, Texture, TextureView,
};

use crate::{
    context::device,
    jobs::frustum_cull::AABB,
    load::CountedBuffer,
    pipeline::{simple, GBuffer, Vertex3D},
    transform::Spatial,
    Transform,
};

use super::Bundle;

/// Renderable that draws a pixelated mesh
pub struct PixelMesh(super::Mesh);

impl PixelMesh {
    /// Create a new renderable from a group of assets
    pub fn new(mesh: Rc<Arc<CountedBuffer>>, tex: Rc<Arc<(Texture, TextureView)>>) -> Self {
        Self(super::Mesh::new(mesh, tex))
    }
}

impl Bundle for PixelMesh {
    fn bundle(&self) -> RenderBundle {
        let mut bundle = device().create_render_bundle_encoder(&RenderBundleEncoderDescriptor {
            label: None,
            color_formats: GBuffer::color_formats(),
            depth_stencil: GBuffer::depth_format(),
            sample_count: 1,
            multiview: None,
        });

        // start recording render commands
        bundle.set_pipeline(&*simple::PIPELINE);
        bundle.set_bind_group(0, &self.0.tex_grp, &[]);
        bundle.set_bind_group(1, &self.0.transform_binding, &[]);
        bundle.set_vertex_buffer(0, self.0.mesh.slice(..));
        bundle.draw(0..self.0.mesh.len(), 0..1);
        bundle.finish(&RenderBundleDescriptor { label: None })
    }
}

impl Spatial for PixelMesh {
    fn transform(&self) -> &Transform {
        self.0.transform()
    }

    fn bound(&self) -> AABB {
        todo!()
    }
}

/// Generate a vertex buffer for a given mesh
pub fn vertex_buffer(mesh: &Mesh<f32>) -> (Vec<u8>, usize) {
    let mut verts: Vec<Vertex3D> = vec![];
    for (a, b, c) in mesh.faces() {
        let gen_vert = |main: Vert, a: Vert, b: Vert, c: Vert| {
            let mut vert = Vertex3D::new(main.pos.x, main.pos.y, main.pos.z);
            if let Some(norm) = main.norm {
                vert = vert.normal(norm.x, norm.y, norm.z);
            }
            if let Some(uv) = main.uv {
                vert = vert.uv(uv.x, uv.y);
            }

            // add info needed for barycentric coords
            vert = vert.local_space(a.pos.into(), b.pos.into(), c.pos.into());
            if let (Some(a), Some(b), Some(c)) = (a.norm, b.norm, c.norm) {
                vert = vert.tri_norm(a.into(), b.into(), c.into());
            }
            if let (Some(a), Some(b), Some(c)) = (a.uv, b.uv, c.uv) {
                vert = vert.tri_uv(a.into(), b.into(), c.into());
            }
            vert
        };

        verts.push(gen_vert(a, a, b, c));
        verts.push(gen_vert(b, a, b, c));
        verts.push(gen_vert(c, a, b, c));
    }

    // create buffer out of vertex list
    let mut buffer = vec![];
    for v in &verts {
        buffer.extend_from_slice(bytemuck::bytes_of(v));
    }
    (buffer, verts.len())
}
