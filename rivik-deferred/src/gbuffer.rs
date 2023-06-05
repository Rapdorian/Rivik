//! Deferred GBuffer

use once_cell::sync::Lazy;
use wgpu::{BindGroupLayout, LoadOp};

use crate::context::device;

const GBUFFER_SIZE: [u32; 2] = [1920, 1080];

/// Create a texture intended to be used in the GBuffer
fn create_texture(name: &str, size: [u32; 2], format: wgpu::TextureFormat) -> wgpu::TextureView {
    let tex = device().create_texture(&wgpu::TextureDescriptor {
        label: Some(name),
        size: wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: Default::default(),
    });

    tex.create_view(&wgpu::TextureViewDescriptor::default())
}

// TODO: Maybe allow resizable gbuffer in the future
pub static DIFFUSE: Lazy<wgpu::TextureView> = Lazy::new(|| {
    create_texture(
        "COLOR GBUFFER",
        GBUFFER_SIZE,
        wgpu::TextureFormat::Rgba8UnormSrgb,
    )
});
pub static NORMAL: Lazy<wgpu::TextureView> = Lazy::new(|| {
    create_texture(
        "NORMAL GBUFFER",
        GBUFFER_SIZE,
        wgpu::TextureFormat::Rg16Float,
    )
});
pub static MATERIAL: Lazy<wgpu::TextureView> = Lazy::new(|| {
    create_texture(
        "MATERIAL GBUFFER",
        GBUFFER_SIZE,
        //wgpu::TextureFormat::Rgba8UnormSrgb,
        wgpu::TextureFormat::Rgba16Float,
    )
});
pub static DEPTH: Lazy<wgpu::TextureView> = Lazy::new(|| {
    create_texture(
        "DEPTH GBUFFER",
        GBUFFER_SIZE,
        wgpu::TextureFormat::Depth24Plus,
    )
});

pub static HDR: Lazy<wgpu::TextureView> = Lazy::new(|| {
    create_texture(
        "HDR GBUFFER",
        GBUFFER_SIZE,
        wgpu::TextureFormat::Rgba16Float,
    )
});

pub const TARGETS: &[Option<wgpu::ColorTargetState>] = &[
    Some(wgpu::ColorTargetState {
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        blend: Some(wgpu::BlendState::REPLACE),
        write_mask: wgpu::ColorWrites::ALL,
    }),
    Some(wgpu::ColorTargetState {
        format: wgpu::TextureFormat::Rg16Float,
        blend: Some(wgpu::BlendState::REPLACE),
        write_mask: wgpu::ColorWrites::ALL,
    }),
    Some(wgpu::ColorTargetState {
        //format: wgpu::TextureFormat::Rgba8UnormSrgb,
        format: wgpu::TextureFormat::Rgba16Float,
        blend: Some(wgpu::BlendState::REPLACE),
        write_mask: wgpu::ColorWrites::ALL,
    }),
];

pub const DEPTH_TARGET: Lazy<wgpu::DepthStencilState> = Lazy::new(|| wgpu::DepthStencilState {
    format: wgpu::TextureFormat::Depth24Plus,
    depth_write_enabled: true,
    depth_compare: wgpu::CompareFunction::LessEqual,
    stencil: Default::default(),
    bias: Default::default(),
});

pub fn color_attachments(
    clear: Option<wgpu::Color>,
) -> Vec<Option<wgpu::RenderPassColorAttachment<'static>>> {
    // determine buffer load setting from clear
    let load = clear
        .map(|color| LoadOp::Clear(color))
        .unwrap_or(LoadOp::Load);

    vec![
        Some(wgpu::RenderPassColorAttachment {
            view: &*DIFFUSE,
            resolve_target: None,
            ops: wgpu::Operations { load, store: true },
        }),
        Some(wgpu::RenderPassColorAttachment {
            view: &*NORMAL,
            resolve_target: None,
            ops: wgpu::Operations { load, store: true },
        }),
        Some(wgpu::RenderPassColorAttachment {
            view: &*MATERIAL,
            resolve_target: None,
            ops: wgpu::Operations { load, store: true },
        }),
    ]
}

pub fn depth_attachment(clear: bool) -> wgpu::RenderPassDepthStencilAttachment<'static> {
    wgpu::RenderPassDepthStencilAttachment {
        view: &*DEPTH,
        depth_ops: Some(wgpu::Operations {
            load: if clear {
                LoadOp::Clear(1.0)
            } else {
                LoadOp::Load
            },
            store: true,
        }),
        stencil_ops: None,
    }
}

/// Bind group layout for writing to the GBuffer
pub static LAYOUT: Lazy<wgpu::BindGroupLayout> = Lazy::new(|| {
    //
    fn entry(num: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: num,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        }
    }

    device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("GBuffer bind group layout"),
        entries: &[
            // color
            entry(0),
            //normal
            entry(1),
            //material
            entry(2),
            // epth
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
        ],
    })
});

pub static BIND: Lazy<wgpu::BindGroup> = Lazy::new(|| {
    fn entry<'a>(num: u32, tex: &'a wgpu::TextureView) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding: num,
            resource: wgpu::BindingResource::TextureView(tex),
        }
    }
    device().create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("GBuffer"),
        layout: &*LAYOUT,
        entries: &[
            entry(0, &DIFFUSE),
            entry(1, &NORMAL),
            entry(2, &MATERIAL),
            entry(3, &*DEPTH),
        ],
    })
});
