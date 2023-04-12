use std::{borrow::Borrow, rc::Rc, sync::Arc};

use assets::formats::mesh::{Mesh, Vert};
use wgpu::{
    AddressMode, BindingResource, FilterMode, RenderBundle, RenderBundleDescriptor,
    RenderBundleEncoderDescriptor, Texture, TextureViewDescriptor,
};

use crate::{
    context::device,
    load::CountedBuffer,
    pipeline::{simple, GBuffer, Vertex3D},
    transform::{self, Spatial},
    Transform,
};

/// I need to create a wrapper type around RenderBundle that also holds references to it's GPU assets
///
/// TODO: Deep dive on when things are freed and how to minimally ensure asset lifetimes
pub struct PixelMesh {
    bundle: RenderBundle,
    transform: Transform,
}

impl PixelMesh {
    /// Create a new renderable from a group of assets
    pub fn new(
        mesh: Rc<Arc<CountedBuffer>>,
        transform: Transform,
        tex: Rc<Arc<(Texture, TextureViewDescriptor)>>,
    ) -> Self {
        // create render bundle for this asset
        let device = device();
        let mut bundle = device.create_render_bundle_encoder(&RenderBundleEncoderDescriptor {
            label: None,
            color_formats: GBuffer::color_formats(),
            depth_stencil: GBuffer::depth_format(),
            sample_count: 1,
            multiview: None,
        });

        // create bind group for uniform buffer
        let uniform = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: transform::layout(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transform.buffer().as_entire_binding(),
            }],
            label: None,
        });

        // create texture bind group
        let texture = tex.0.create_view(&tex.1);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            ..Default::default()
        });

        let texture_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &*simple::TEX_LAYOUT,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&texture),
                },
            ],
        });

        // start recording render commands
        bundle.set_pipeline(&*simple::PIPELINE);
        bundle.set_bind_group(0, &texture_group, &[]);
        bundle.set_bind_group(1, &uniform, &[]);
        bundle.set_vertex_buffer(0, mesh.slice(..));
        bundle.draw(0..mesh.len(), 0..1);
        let bundle = bundle.finish(&RenderBundleDescriptor { label: None });
        Self { bundle, transform }
    }
}

impl Borrow<RenderBundle> for PixelMesh {
    fn borrow(&self) -> &RenderBundle {
        &self.bundle
    }
}

impl Spatial for PixelMesh {
    fn transform(&self) -> &Transform {
        &self.transform
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

// /// Contains references to a mesh that has been uploaded to the GPU
// pub struct Mesh {
//     pub(crate) vbuf: Buffer,
//     pub(crate) ibuf: Buffer,
//     pub(crate) icount: u32,
//     pub(crate) tex: wgpu::BindGroup,
// }

// /// Contains a user configurable description of a mesh
// pub struct MeshDescriptor {
//     pub(crate) verts: Vec<Vertex3D>,
//     pub(crate) tris: Vec<u16>,
//     pub(crate) tex: Option<DynamicImage>,
// }

// impl MeshDescriptor {
//     /// Creates an empty mesh
//     pub fn new() -> Self {
//         Self {
//             verts: vec![],
//             tris: vec![],
//             tex: None,
//         }
//     }
//
//     /// Set the diffuse texture of this mesh
//     pub fn set_diffuse(&mut self, diffuse: DynamicImage) {
//         self.tex = Some(diffuse);
//     }
//
//     /// Add a triangle to the mesh
//     pub fn push(&mut self, a: Vertex3D, b: Vertex3D, c: Vertex3D) {
//         let i = self.verts.len() as u16;
//         self.verts.push(a);
//         self.verts.push(b);
//         self.verts.push(c);
//
//         self.tris.push(i);
//         self.tris.push(i + 1);
//         self.tris.push(i + 2);
//     }
//
//     /// Get all vertices as a byte array
//     pub fn vert_data(&self) -> Vec<u8> {
//         let mut data = vec![];
//         for v in &self.verts {
//             data.extend_from_slice(&v.bytes());
//         }
//         data
//     }
//
//     /// Get all triangle indices as a byte array
//     pub fn index_data(&self) -> Vec<u8> {
//         let mut data = vec![];
//         for i in &self.tris {
//             data.extend_from_slice(&i.to_ne_bytes());
//         }
//         data
//     }
// }
