use std::{mem, num::NonZeroU64};

use glam::Mat4;
use once_cell::sync::Lazy;
use rivik_render_types::Model;

use crate::context::device;

pub static LAYOUT: Lazy<wgpu::BindGroupLayout> = Lazy::new(|| {
    fn entry(num: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: num,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }
    }

    device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Material Layout"),
        entries: &[
            // color
            entry(0),
            // roughness,
            entry(1),
            // metalness,
            entry(2),
            // normal
            entry(3),
        ],
    })
});

pub fn bind(
    textures: &[(wgpu::Texture, wgpu::TextureView)],
    material: &Model<u32>,
) -> wgpu::BindGroup {
    let entry = |num: u32, tex: u32| -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: num,
            resource: wgpu::BindingResource::TextureView(&textures[tex as usize].1),
        }
    };

    device().create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &*LAYOUT,
        entries: &[
            entry(0, material.diffuse),
            entry(1, material.rough),
            entry(2, material.metal),
            entry(3, material.normal),
        ],
    })
}
