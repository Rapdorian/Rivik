use std::{mem, num::NonZeroU64};

use glam::{Mat4, Vec3};
use once_cell::sync::Lazy;

use crate::context::device;

pub static LAYOUT: Lazy<wgpu::BindGroupLayout> = Lazy::new(|| {
    device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Directional Light buffer Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
});
