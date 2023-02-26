use std::borrow::Borrow;

use ultraviolet::Vec3;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, BufferBinding, BufferUsages,
    RenderBundle, RenderBundleDescriptor, RenderBundleEncoderDescriptor,
};

use crate::{
    context::{device, gbuffer, queue, surface_config},
    pipeline::{sun, GBuffer},
};

pub struct SunLight {
    bundle: RenderBundle,
    buffer: Buffer,
}

impl SunLight {
    /// Creates a new ambient light on the GPU
    pub fn new(color: Vec3, direction: Vec3) -> Self {
        // create uniform buffer
        let device = device();

        let mut buffer = vec![];
        buffer.extend_from_slice(color.as_byte_slice());
        buffer.extend_from_slice(&1.0_f32.to_le_bytes());
        buffer.extend_from_slice(direction.as_byte_slice());
        buffer.extend_from_slice(&0.0_f32.to_le_bytes());

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &buffer,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // create render bundle for this light
        let gbuffer = gbuffer();

        let mut bundle = device.create_render_bundle_encoder(&RenderBundleEncoderDescriptor {
            label: None,
            color_formats: &[Some(GBuffer::hdr_format())],
            depth_stencil: None,
            sample_count: 1,
            multiview: None,
        });

        let uniform_buffer = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: sun::layout(),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        // record draw commands to render bundle
        bundle.set_pipeline(sun::pipeline());
        bundle.set_bind_group(0, &gbuffer.bind_group, &[]);
        bundle.set_bind_group(1, &uniform_buffer, &[]);
        bundle.draw(0..7, 0..1);

        let bundle = bundle.finish(&RenderBundleDescriptor { label: None });
        Self { buffer, bundle }
    }

    /// Set the direction of this sun light
    pub fn set_direction(&self, direction: Vec3) {
        queue().write_buffer(&self.buffer, 16, direction.as_byte_slice());
    }

    /// Set the color of this sun light
    pub fn set_color(&self, color: Vec3) {
        queue().write_buffer(&self.buffer, 0, color.as_byte_slice());
    }
}

impl Borrow<RenderBundle> for SunLight {
    fn borrow(&self) -> &RenderBundle {
        &self.bundle
    }
}
