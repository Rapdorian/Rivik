use once_cell::sync::Lazy;
use wgpu::{BindGroupLayout, RenderPipeline};

use crate::{context::device, shader, transform};

use super::{vertex3d::Vertex3D, GBuffer};

pub static PIPELINE: Lazy<RenderPipeline> = Lazy::new(|| {
    GBuffer::geom_pipeline(
        &shader!("../shaders/simple3d.wgsl").unwrap(),
        &[&*TEX_LAYOUT, transform::layout()],
        Vertex3D::LAYOUT,
    )
});

pub static TEX_LAYOUT: Lazy<BindGroupLayout> = Lazy::new(|| {
    device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
    })
});
