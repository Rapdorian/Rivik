//! Utilities for rendering a static mesh
use std::{borrow::Borrow, rc::Rc, sync::Arc};

use assets::formats::{self, mesh::Vert};
use wgpu::{
    RenderBundle, RenderBundleDescriptor, RenderBundleEncoderDescriptor, Texture, TextureView,
};

use crate::{
    context::device,
    load::CountedBuffer,
    pipeline::{
        mesh::{self, MeshVertex},
        simple, GBuffer,
    },
    transform::{self, Spatial},
    Transform,
};

/// Basic mesh renderable
pub struct Mesh {
    bundle: RenderBundle,
    transform: Transform,

    //keep the following assets alive
    mesh: Rc<Arc<CountedBuffer>>,
    tex: Rc<Arc<(Texture, TextureView)>>,
}

impl Borrow<RenderBundle> for Mesh {
    fn borrow(&self) -> &RenderBundle {
        &self.bundle
    }
}

impl Spatial for Mesh {
    fn transform(&self) -> &Transform {
        &self.transform
    }
}

/// TODO: I need to change this so it doesn't use the same barycentric coord stuf that PixelMesh
/// needs.
impl Mesh {
    /// Create a new mesh renderable
    pub fn new(mesh: Rc<Arc<CountedBuffer>>, tex: Rc<Arc<(Texture, TextureView)>>) -> Self {
        let device = device();
        let mut bundle = device.create_render_bundle_encoder(&RenderBundleEncoderDescriptor {
            label: None,
            color_formats: GBuffer::color_formats(),
            depth_stencil: GBuffer::depth_format(),
            sample_count: 1,
            multiview: None,
        });

        let transform = Transform::default();
        let transform_binding = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: transform::layout(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transform.buffer().as_entire_binding(),
            }],
            label: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            ..Default::default()
        });
        let texture_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &*simple::TEX_LAYOUT,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&tex.1),
                },
            ],
        });

        // start recording render commands
        bundle.set_pipeline(&*mesh::PIPELINE);
        bundle.set_bind_group(0, &texture_group, &[]);
        bundle.set_bind_group(1, &transform_binding, &[]);
        bundle.set_vertex_buffer(0, mesh.slice(..));
        bundle.draw(0..mesh.len(), 0..1);
        let bundle = bundle.finish(&RenderBundleDescriptor { label: None });
        Self {
            bundle,
            transform,
            mesh,
            tex,
        }
    }
}

/// Generate a vertex buffer for a given mesh
pub fn vertex_buffer(mesh: &formats::mesh::Mesh<f32>) -> (Vec<u8>, usize) {
    let mut verts: Vec<MeshVertex> = vec![];
    for (a, b, c) in mesh.faces() {
        let gen_vert = |v: Vert| {
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

    // create buffer out of vertex list
    let mut buffer = vec![];
    for v in &verts {
        buffer.extend_from_slice(bytemuck::bytes_of(v));
    }
    (buffer, verts.len())
}
