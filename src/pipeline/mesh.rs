use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};
use once_cell::sync::OnceCell;
use wgpu::{
    BlendState, ColorTargetState, ColorWrites, DepthStencilState, RenderPipeline, TextureFormat,
};
use wgpu_macros::VertexLayout;

use crate::{context::device, shader, transform};

use super::simple;

static PIPELINE: OnceCell<RenderPipeline> = OnceCell::new();

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod, VertexLayout, Default)]
pub struct MeshVertex {
    pub pos: [f32; 3],
    pub norm: [f32; 3],
    pub uv: [f32; 2],
}

pub fn pipeline() -> &'static RenderPipeline {
    if let Some(pipeline) = PIPELINE.get() {
        pipeline
    } else {
        // generate new pipeline
        let shader = device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
                &shader!("../shaders/mesh.wgsl").unwrap(),
            )),
        });

        let layout = device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[simple::texture_layout(), transform::layout()],
            push_constant_ranges: &[],
        });

        let pipeline = device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[MeshVertex::LAYOUT],
            },
            primitive: Default::default(),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                // TODO: This is a good canidate for making a method on gbuffer
                targets: &[
                    Some(ColorTargetState {
                        format: TextureFormat::Rgba16Float,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::default(),
                    }),
                    Some(ColorTargetState {
                        format: TextureFormat::Rgba16Float,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::default(),
                    }),
                    Some(ColorTargetState {
                        format: TextureFormat::Rgba16Float,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::default(),
                    }),
                ],
            }),
            multiview: None,
        });
        PIPELINE.try_insert(pipeline).unwrap()
    }
}
