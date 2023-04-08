use std::{borrow::Cow, num::NonZeroU64};

use once_cell::sync::OnceCell;
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer,
    BufferBindingType, ColorTargetState, ColorWrites, FragmentState, MultisampleState,
    PipelineLayoutDescriptor, PrimitiveState, RenderPipeline, RenderPipelineDescriptor,
    ShaderStages, VertexState,
};

use crate::{
    context::{device, gbuffer, surface_config},
    shader, transform,
};

use super::{GBuffer, LIGHT_BLEND};

static SUN_PIPE: OnceCell<RenderPipeline> = OnceCell::new();
static SUN_LAYOUT: OnceCell<BindGroupLayout> = OnceCell::new();

/// Fetch the uniform layout of the sun light pipeline
pub fn layout() -> &'static BindGroupLayout {
    if let Some(layout) = SUN_LAYOUT.get() {
        layout
    } else {
        let _ = pipeline();
        layout()
    }
}

/// Fetch the sun light pipeline
pub fn pipeline() -> &'static RenderPipeline {
    if let Some(pipe) = SUN_PIPE.get() {
        pipe
    } else {
        SUN_PIPE.try_insert(create_pipeline()).unwrap()
    }
}

fn create_pipeline() -> RenderPipeline {
    let device = device();
    let gbuffer = gbuffer();

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader!("../shaders/sun.wgsl").unwrap())),
    });

    let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZeroU64::new(32),
            },
            count: None,
        }],
    });

    let bind_group_layout = SUN_LAYOUT.try_insert(bind_group_layout).unwrap();

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&gbuffer.layout, bind_group_layout, transform::layout()],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(ColorTargetState {
                format: GBuffer::hdr_format(),
                blend: Some(LIGHT_BLEND),
                write_mask: ColorWrites::ALL,
            })],
        }),
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
        multiview: None,
    })
}
