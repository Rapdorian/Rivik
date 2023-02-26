use std::{borrow::Cow, num::NonZeroU64};

use once_cell::sync::OnceCell;
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType::{self, Buffer},
    BlendComponent, BlendFactor, BlendOperation, BlendState, ColorTargetState, ColorWrites, Device,
    PushConstantRange, RenderPipeline, SamplerBindingType, ShaderStages, TextureFormat,
    TextureSampleType, TextureViewDimension,
};

use crate::{
    context::{device, gbuffer, surface_config},
    shader,
};

use super::{GBuffer, LIGHT_BLEND};

static DISPLAY_PIPE: OnceCell<RenderPipeline> = OnceCell::new();
static DISPLAY_LAYOUT: OnceCell<BindGroupLayout> = OnceCell::new();

pub fn layout() -> &'static BindGroupLayout {
    if let Some(layout) = DISPLAY_LAYOUT.get() {
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
    let fmt = surface_config().read().unwrap().format;
    if let Some(pipe) = DISPLAY_PIPE.get() {
        pipe
    } else {
        DISPLAY_PIPE
            .try_insert(output_pipeline(device, fmt, gbuffer))
            .unwrap()
    }
}

fn output_pipeline(device: &Device, fmt: TextureFormat, gbuffer: &GBuffer) -> RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
            &shader!("../shaders/display.wgsl").unwrap(),
        )),
    });

    let hdr_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("HDR buffer"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    multisampled: false,
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
        ],
    });
    let hdr_layout = DISPLAY_LAYOUT.try_insert(hdr_layout).unwrap();

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[hdr_layout],
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
