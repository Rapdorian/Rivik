/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! A mesh type for drawing a single static mesh
//! TODO: This should be expanded to models with multiple meshes

use std::{rc::Rc, sync::Arc};

use assets::formats::{self, mesh::Vert};
use glam::Vec3A;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, RenderBundle, RenderBundleDescriptor,
    RenderBundleEncoderDescriptor, Texture, TextureView,
};

use crate::{
    context::device,
    jobs::frustum_cull::AABB,
    load::CountedBuffer,
    pipeline::{
        mesh::{self, MeshVertex},
        simple, GBuffer,
    },
    sampler,
    transform::Spatial,
    Transform,
};

use super::Bundle;
/// Draw a mesh
pub struct Mesh {
    pub(super) transform: Transform,
    pub(super) mesh: Rc<Arc<CountedBuffer>>,
    pub(super) tex: Rc<Arc<(Texture, TextureView)>>,
    pub(super) tex_grp: BindGroup,
    pub(super) transform_binding: BindGroup,
}

impl Spatial for Mesh {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn bound(&self) -> AABB {
        self.mesh.bounds()
    }
}

impl Mesh {
    /// Create a drawable
    pub fn new(mesh: Rc<Arc<CountedBuffer>>, tex: Rc<Arc<(Texture, TextureView)>>) -> Self {
        let transform = Transform::default();

        let texture_group = device().create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &*simple::TEX_LAYOUT,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler::PIXEL),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&tex.1),
                },
            ],
        });

        Self {
            mesh,
            tex,
            tex_grp: texture_group,
            transform_binding: transform.binding(),
            transform,
        }
    }
}

impl Bundle for Mesh {
    /// Bundle the draw commands for this object
    fn bundle(&self) -> RenderBundle {
        let mut bundle = device().create_render_bundle_encoder(&RenderBundleEncoderDescriptor {
            label: None,
            color_formats: GBuffer::color_formats(),
            depth_stencil: GBuffer::depth_format(),
            sample_count: 1,
            multiview: None,
        });

        bundle.set_pipeline(&*mesh::PIPELINE);
        bundle.set_bind_group(0, &self.tex_grp, &[]);
        bundle.set_bind_group(1, &self.transform_binding, &[]);
        bundle.set_vertex_buffer(0, self.mesh.slice(..));
        bundle.draw(0..self.mesh.len(), 0..1);

        bundle.finish(&RenderBundleDescriptor { label: None })
    }
}

/// Generate a vertex buffer for a given mesh
pub fn vertex_buffer(mesh: &formats::mesh::Mesh<f32>) -> (Vec<u8>, usize, AABB) {
    let mut verts: Vec<MeshVertex> = vec![];

    let mut min = Vec3A::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
    let mut max = Vec3A::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);

    for (a, b, c) in mesh.faces() {
        let mut gen_vert = |v: Vert| {
            // find the bounding box of these verts
            if v.pos.x < min.x {
                min.x = v.pos.x;
            } else if v.pos.x > max.x {
                max.x = v.pos.x;
            }
            if v.pos.y < min.y {
                min.y = v.pos.y;
            } else if v.pos.y > max.y {
                max.y = v.pos.y;
            }
            if v.pos.z < min.z {
                min.z = v.pos.z;
            } else if v.pos.z > max.z {
                max.z = v.pos.z;
            }

            let norm = v.norm.unwrap_or(mint::Point3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            });
            let uv = v.uv.unwrap_or(mint::Point2 { x: 0.0, y: 0.0 });
            MeshVertex {
                pos: [v.pos.x, v.pos.y, v.pos.z],
                norm: [norm.x, norm.y, norm.z],
                uv: [uv.x, uv.y],
            }
        };
        verts.push(gen_vert(a));
        verts.push(gen_vert(b));
        verts.push(gen_vert(c));
    }

    let mut aabb = AABB::default();
    aabb.min(min);
    aabb.max(max);

    // create buffer out of vertex list
    let mut buffer = vec![];
    for v in &verts {
        buffer.extend_from_slice(bytemuck::bytes_of(v));
    }
    (buffer, verts.len(), aabb)
}
