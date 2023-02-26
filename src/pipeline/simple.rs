use std::{borrow::Cow, num::NonZeroU64};

use once_cell::sync::OnceCell;
use wgpu::{
    BindGroupLayout, BlendState, ColorTargetState, ColorWrites, DepthStencilState, Device,
    RenderPipeline, TextureFormat,
};

use crate::{context::device, shader};

use super::vertex3d::Vertex3D;

// pipelines will be managed statically
static SIMPLE_PIPE: OnceCell<Simple3DPipeline> = OnceCell::new();

fn inner() -> &'static Simple3DPipeline {
    if let Some(pipeline) = SIMPLE_PIPE.get() {
        pipeline
    } else {
        // create new pipeline and insert it
        SIMPLE_PIPE.try_insert(simple3d_pipeline(device())).unwrap()
    }
}

pub fn pipeline() -> &'static RenderPipeline {
    &inner().pipeline
}

pub fn layout() -> &'static BindGroupLayout {
    &inner().proj_layout
}

pub fn texture_layout() -> &'static BindGroupLayout {
    &inner().tex_layout
}

#[derive(Debug)]
pub struct Simple3DPipeline {
    pub(crate) pipeline: RenderPipeline,
    pub(crate) proj_layout: BindGroupLayout,
    //pub(crate) uniform_layout: BindGroupLayout,
    pub(crate) tex_layout: BindGroupLayout,
}

fn simple3d_pipeline(device: &Device) -> Simple3DPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
            &shader!("../shaders/simple3d.wgsl").unwrap(),
        )),
    });

    let proj_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZeroU64::new(64 * 3),
            },
            count: None,
        }],
    });

    let tex_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                // This should match the filterable field of the
                // corresponding Texture entry above.
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&tex_layout, &proj_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex3D::LAYOUT],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
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
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth24Plus,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });
    Simple3DPipeline {
        pipeline: render_pipeline,
        proj_layout,
        tex_layout,
    }
}
