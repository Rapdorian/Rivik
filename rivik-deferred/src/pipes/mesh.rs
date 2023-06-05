//! Create a render pipeline for a basic PBR mesh render

use std::borrow::Cow;

use once_cell::sync::Lazy;

use crate::{
    context::device,
    gbuffer,
    inputs::{material, sampler, transform, vertex},
    shader,
};

pub static MESH_PIPE: Lazy<wgpu::RenderPipeline> = Lazy::new(|| {
    // Create the pipeline layout
    let layout = device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Geom Render Pipeline layout"),
        bind_group_layouts: &[&*sampler::LAYOUT, &*transform::LAYOUT, &*material::LAYOUT],
        push_constant_ranges: &[],
    });

    let shader = shader!("mesh.wgsl").unwrap();

    device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Geom Render Pipeline"),
        layout: Some(&layout),
        primitive: wgpu::PrimitiveState::default(),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[vertex::LAYOUT.clone()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &*gbuffer::TARGETS,
        }),
        depth_stencil: Some(gbuffer::DEPTH_TARGET.clone()),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    })
});
