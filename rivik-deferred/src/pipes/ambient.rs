use once_cell::sync::Lazy;

use crate::{
    context::{device, surface_config},
    gbuffer,
    inputs::sampler,
    shader,
};

use super::LIGHT_BLEND;

pub static PIPE: Lazy<wgpu::RenderPipeline> = Lazy::new(|| {
    let layout = device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Ambient pipeline layout"),
        bind_group_layouts: &[&*sampler::LAYOUT, &*gbuffer::LAYOUT],
        push_constant_ranges: &[wgpu::PushConstantRange {
            stages: wgpu::ShaderStages::FRAGMENT,
            range: 0..(4 * 4),
        }],
    });

    let shader = shader!("ambient.wgsl").unwrap();

    let fmt = surface_config().format;

    device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Ambient pipeline"),
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
