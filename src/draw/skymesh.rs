use std::{borrow::Borrow, rc::Rc, sync::Arc};

use wgpu::{
    RenderBundle, RenderBundleDescriptor, RenderBundleEncoderDescriptor, Texture,
    TextureViewDescriptor,
};

use crate::{
    context::device,
    load::CountedBuffer,
    pipeline::{simple, sky_box, GBuffer},
    transform::{self, Spatial},
    Transform,
};

pub struct SkyMesh {
    bundle: RenderBundle,
    transform: Transform,
}

impl Borrow<RenderBundle> for SkyMesh {
    fn borrow(&self) -> &RenderBundle {
        &self.bundle
    }
}

impl Spatial for SkyMesh {
    fn transform(&self) -> &Transform {
        &self.transform
    }
}

impl SkyMesh {
    pub fn new(
        mesh: Rc<Arc<CountedBuffer>>,
        tex: Rc<Arc<(Texture, TextureViewDescriptor)>>,
    ) -> Self {
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

        let texture = tex.0.create_view(&tex.1);
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
                    resource: wgpu::BindingResource::TextureView(&texture),
                },
            ],
        });

        // start recording render commands
        bundle.set_pipeline(&*sky_box::PIPELINE);
        bundle.set_bind_group(0, &texture_group, &[]);
        bundle.set_bind_group(1, &transform_binding, &[]);
        bundle.set_vertex_buffer(0, mesh.slice(..));
        bundle.draw(0..mesh.len(), 0..1);
        let bundle = bundle.finish(&RenderBundleDescriptor { label: None });
        Self { bundle, transform }
    }
}
