//! A collection of common sampler types

use once_cell::sync::Lazy;
use wgpu::{AddressMode, FilterMode, Sampler, SamplerDescriptor};

use crate::context::device;

pub static LAYOUT: Lazy<wgpu::BindGroupLayout> = Lazy::new(|| {
    fn sampler(num: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: num,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        }
    }

    device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Global Variables Layout"),
        entries: &[sampler(0)],
    })
});

pub static BIND: Lazy<wgpu::BindGroup> = Lazy::new(|| {
    fn sampler<'a>(num: u32, smplr: &'a Sampler) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding: num,
            resource: wgpu::BindingResource::Sampler(smplr),
        }
    }

    device().create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Globals"),
        layout: &*LAYOUT,
        entries: &[sampler(0, &DEFAULT)],
    })
});

/// Nearest Neighbour texture sampler
pub const PIXEL: Lazy<Sampler> = Lazy::new(|| {
    device().create_sampler(&SamplerDescriptor {
        label: Some("Nearest Neighbour Sampler"),
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        ..Default::default()
    })
});

/// Default sampler for WGPU
pub const DEFAULT: Lazy<Sampler> = Lazy::new(|| {
    device().create_sampler(&SamplerDescriptor {
        label: Some("WGPU Default Sampler"),
        ..Default::default()
    })
});

/// Bilinear texture sampler
pub const LINEAR: Lazy<Sampler> = Lazy::new(|| {
    device().create_sampler(&SamplerDescriptor {
        label: Some("Linear Sampler"),
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        ..Default::default()
    })
});

/// Clamped nearest neighbour sampler
pub const CLAMP_PIXEL: Lazy<Sampler> = Lazy::new(|| {
    device().create_sampler(&SamplerDescriptor {
        label: Some("Clamped Nearest Neighbour Sampler"),
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        ..Default::default()
    })
});

/// Clamped bilinear texture sampler
pub const CLAMP_LINEAR: Lazy<Sampler> = Lazy::new(|| {
    device().create_sampler(&SamplerDescriptor {
        label: Some("Clamped Linear Sampler"),
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        ..Default::default()
    })
});
