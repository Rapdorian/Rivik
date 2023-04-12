use std::{borrow::Borrow, ops::Deref};

use crate::{
    context::{device, gbuffer, queue, surface_config},
    pipeline::{ambient, GBuffer},
    transform::Spatial,
    Transform,
};
use ultraviolet::{Vec3, Vec4};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, BufferBinding, BufferUsages,
    RenderBundle, RenderBundleDescriptor, RenderBundleEncoderDescriptor,
};

/// Convienience object for handling the uniform buffer of an ambient light
pub struct AmbientLight {
    bundle: RenderBundle,
    buffer: Buffer,
    transform: Transform,
}

impl Spatial for AmbientLight {
    fn transform(&self) -> &Transform {
        &self.transform
    }
}

impl AmbientLight {
    /// Creates a new ambient light on the GPU
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        // create buf
        let device = device();
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: Vec4::new(r, g, b, 1.0).as_byte_slice(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let bundle = ambient_light(&buffer);
        Self {
            bundle,
            buffer,
            transform: Transform::default(),
        }
    }

    /// Queues a write to the internal buffer for this ambient lights color
    pub fn set_color(&self, r: f32, g: f32, b: f32) {
        queue().write_buffer(&self.buffer, 0, Vec4::new(r, g, b, 1.0).as_byte_slice());
    }
}

impl Borrow<RenderBundle> for AmbientLight {
    fn borrow(&self) -> &RenderBundle {
        &self.bundle
    }
}

/// Generates a renderbundle for an ambient light
fn ambient_light(uniform: &Buffer) -> RenderBundle {
    let device = device();
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
        layout: ambient::layout(),
        entries: &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::Buffer(BufferBinding {
                buffer: uniform,
                offset: 0,
                size: None,
            }),
        }],
    });

    //let data = color.as_byte_slice();
    //assert_eq!(12, data.len());

    bundle.set_pipeline(ambient::pipeline());
    bundle.set_bind_group(0, &gbuffer.bind_group, &[]);
    bundle.set_bind_group(1, &uniform_buffer, &[]);
    bundle.draw(0..7, 0..1);

    bundle.finish(&RenderBundleDescriptor { label: None })
}
