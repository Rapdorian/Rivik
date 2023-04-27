/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! Ambient lighting pipeline

use std::{borrow::Cow, num::NonZeroU64};

use once_cell::sync::OnceCell;
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType::Buffer,
    ColorTargetState, ColorWrites, Device, RenderPipeline, ShaderStages, TextureFormat,
};

use crate::{
    context::{device, gbuffer},
    shader,
};

use super::{GBuffer, LIGHT_BLEND};

static AMBIENT_PIPE: OnceCell<RenderPipeline> = OnceCell::new();
static AMBIENT_LAYOUT: OnceCell<BindGroupLayout> = OnceCell::new();

/// input layout for ambient lighting
pub fn layout() -> &'static BindGroupLayout {
    if let Some(layout) = AMBIENT_LAYOUT.get() {
        &layout
    } else {
        let _ = pipeline();
        layout()
    }
}

/// Fetch the ambient light pipeline
pub fn pipeline() -> &'static RenderPipeline {
    let device = device();
    let gbuffer = gbuffer();
    if let Some(pipe) = AMBIENT_PIPE.get() {
        pipe
    } else {
        AMBIENT_PIPE
            .try_insert(output_pipeline(device, GBuffer::hdr_format(), gbuffer))
            .unwrap()
    }
}

fn output_pipeline(device: &Device, fmt: TextureFormat, gbuffer: &GBuffer) -> RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
            &shader!("../shaders/ambient.wgsl").unwrap(),
        )),
    });

    let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZeroU64::new(16),
            },
            count: None,
        }],
    });
    let bind_group_layout = AMBIENT_LAYOUT.try_insert(bind_group_layout).unwrap();

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&gbuffer.layout, bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(ColorTargetState {
                format: fmt,
                blend: Some(LIGHT_BLEND),
                write_mask: ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    })
}
