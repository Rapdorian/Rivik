use std::{mem, num::NonZeroU64};

use glam::Mat4;
use once_cell::sync::Lazy;

use crate::context::device;

pub static LAYOUT: Lazy<wgpu::BindGroupLayout> = Lazy::new(|| {
    device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Transform Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: true,
                min_binding_size: NonZeroU64::new(mem::size_of::<Mat4>() as u64 * 3),
            },
            count: None,
        }],
    })
});
