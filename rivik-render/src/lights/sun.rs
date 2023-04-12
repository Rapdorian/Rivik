use std::borrow::Borrow;

use mint::Vector3;
use ultraviolet::Vec3;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, BufferBinding, BufferUsages,
    RenderBundle, RenderBundleDescriptor, RenderBundleEncoderDescriptor,
};

use crate::{
    context::{device, gbuffer, queue},
    pipeline::{sun, GBuffer},
    transform::{self, Spatial},
    Transform,
};

/// A Directional light that can be rendered to a frame
pub struct SunLight {
    bundle: RenderBundle,
    buffer: Buffer,
    transform: Transform,
}

impl Spatial for SunLight {
    fn transform(&self) -> &Transform {
        &self.transform
    }
}

impl SunLight {
    /// Creates a new ambient light on the GPU
    pub fn new(color: impl Into<Vector3<f32>>, direction: impl Into<Vector3<f32>>) -> Self {
        let color = Vec3::from(color.into());
        let direction = Vec3::from(direction.into());
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

        let transform = Transform::default();

        let t_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: transform::layout(),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: transform.buffer(),
                    offset: 0,
                    size: None,
                }),
            }],
        });

        // record draw commands to render bundle
        bundle.set_pipeline(sun::pipeline());
        bundle.set_bind_group(0, &gbuffer.bind_group, &[]);
        bundle.set_bind_group(1, &uniform_buffer, &[]);
        bundle.set_bind_group(2, &t_group, &[]);
        bundle.draw(0..7, 0..1);

        let bundle = bundle.finish(&RenderBundleDescriptor { label: None });
        Self {
            buffer,
            transform,
            bundle,
        }
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
