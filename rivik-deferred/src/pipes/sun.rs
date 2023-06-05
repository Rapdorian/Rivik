use once_cell::sync::Lazy;

use crate::{
    context::{device, surface_config},
    gbuffer,
    inputs::{sampler, sun_light},
    shader,
};

use super::LIGHT_BLEND;

pub static PIPE: Lazy<wgpu::RenderPipeline> = Lazy::new(|| {
    let layout = device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Directional Light pipeline layout"),
        bind_group_layouts: &[&*sampler::LAYOUT, &*gbuffer::LAYOUT, &*sun_light::LAYOUT],
        push_constant_ranges: &[],
    });

    let shader = shader!("sun.wgsl").unwrap();

    let fmt = surface_config().format;

    device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Directional Light pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: fmt,
                blend: Some(LIGHT_BLEND),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: Default::default(),
        multisample: Default::default(),
        depth_stencil: None,
        multiview: None,
    })
});

pub static PIPE_HDR: Lazy<wgpu::RenderPipeline> = Lazy::new(|| {
    let layout = device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Directional Light pipeline layout"),
        bind_group_layouts: &[&*sampler::LAYOUT, &*gbuffer::LAYOUT, &*sun_light::LAYOUT],
        push_constant_ranges: &[],
    });

    let shader = shader!("sun.wgsl").unwrap();

    device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Directional Light pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba16Float,
                blend: Some(LIGHT_BLEND),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: Default::default(),
        multisample: Default::default(),
        depth_stencil: None,
        multiview: None,
    })
});
