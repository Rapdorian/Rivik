/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::borrow::Borrow;

use wgpu::{
    AddressMode, FilterMode, RenderBundle, RenderBundleDescriptor, RenderBundleEncoderDescriptor,
};

use crate::{
    context::{device, gbuffer, surface_config},
    pipeline::display,
};

/// Convienience object for drawing a buffer to screen
///
/// TODO: Allow changing buffer displayed
/// TODO: Allow settings camera options for HDR
///     TODO: Research into how this should be handled
pub struct DisplayFilter {
    bundle: RenderBundle,

    /// This buffer is not really used yet but will be
    #[allow(dead_code)]
    hdr: wgpu::BindGroup,
}

impl DisplayFilter {
    /// Creates a new ambient light on the GPU
    pub fn new() -> Self {
        // create buf
        let device = device();
        let gbuffer = gbuffer();
        let fmt = surface_config().read().unwrap().format;

        let mut bundle = device.create_render_bundle_encoder(&RenderBundleEncoderDescriptor {
            label: None,
            color_formats: &[Some(fmt)],
            depth_stencil: None,
            sample_count: 1,
            multiview: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            ..Default::default()
        });

        let hdr = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: display::layout(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&gbuffer.hdr_view),
                },
            ],
        });

        bundle.set_pipeline(display::pipeline());
        bundle.set_bind_group(0, &hdr, &[]);
        bundle.draw(0..7, 0..1);

        let bundle = bundle.finish(&RenderBundleDescriptor { label: None });

        Self { bundle, hdr }
    }

    pub(crate) fn bundle(&self) -> &RenderBundle {
        &self.bundle
    }
}

impl Default for DisplayFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl Borrow<RenderBundle> for DisplayFilter {
    fn borrow(&self) -> &RenderBundle {
        &self.bundle
    }
}
