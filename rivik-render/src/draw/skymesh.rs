/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::{rc::Rc, sync::Arc};

use glam::Vec3A;
use wgpu::{
    RenderBundle, RenderBundleDescriptor, RenderBundleEncoderDescriptor, Texture, TextureView,
};

use crate::{
    context::device,
    jobs::frustum_cull::AABB,
    load::CountedBuffer,
    pipeline::{simple, sky_box, GBuffer},
    sampler,
    transform::{self, Spatial},
    Transform,
};

use super::Bundle;

/// A Unlit mesh with no depth intended to be used for drawing skyboxes
pub struct SkyMesh {
    transform: Transform,
    mesh: Rc<Arc<CountedBuffer>>,
    tex: Rc<Arc<(Texture, TextureView)>>,
}

impl Spatial for SkyMesh {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn bound(&self) -> AABB {
        let mut aabb = AABB::default();
        aabb.min(Vec3A::splat(f32::NEG_INFINITY));
        aabb.max(Vec3A::splat(f32::INFINITY));
        aabb
    }
}

impl SkyMesh {
    /// Create a new skymesh
    pub fn new(mesh: Rc<Arc<CountedBuffer>>, tex: Rc<Arc<(Texture, TextureView)>>) -> Self {
        let transform = Transform::default();

        Self {
            transform,
            mesh,
            tex,
        }
    }
}

impl Bundle for SkyMesh {
    fn bundle(&self) -> RenderBundle {
        let mut bundle = device().create_render_bundle_encoder(&RenderBundleEncoderDescriptor {
            label: None,
            color_formats: GBuffer::color_formats(),
            depth_stencil: GBuffer::depth_format(),
            sample_count: 1,
            multiview: None,
        });

        let transform_binding = device().create_bind_group(&wgpu::BindGroupDescriptor {
            layout: transform::layout(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.transform.buffer().as_entire_binding(),
            }],
            label: None,
        });
        let texture_group = device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &*simple::TEX_LAYOUT,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler::LINEAR),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.tex.1),
                },
            ],
        });

        // start recording render commands
        bundle.set_pipeline(&*sky_box::PIPELINE);
        bundle.set_bind_group(0, &texture_group, &[]);
        bundle.set_bind_group(1, &transform_binding, &[]);
        bundle.set_vertex_buffer(0, self.mesh.slice(..));
        bundle.draw(0..self.mesh.len(), 0..1);
        bundle.finish(&RenderBundleDescriptor { label: None })
    }
}
