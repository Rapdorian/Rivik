//! WGPU backed renderer structure
#![deny(missing_docs)]
#![deny(macro_use_extern_crate)]
#![warn(unused_crate_dependencies)]
#![deny(unused_import_braces)]
#![deny(unused_qualifications)]
#![deny(unused_results)]
#![deny(unused_imports)]
#![warn(variant_size_differences)]

pub mod context;
mod frame;
pub mod tracing;
pub mod transform;
pub use transform::Transform;

/// Contains asset loader functions for fetching GPU assets from disk formats
pub mod load {
    mod mesh;
    mod tex;

    pub use mesh::*;
    pub use tex::*;
}

/// Contains render bundle creation methods for drawing geometry
pub mod draw {
    pub mod mesh;
    pub mod pixel_mesh;
    mod skymesh;

    pub use mesh::Mesh;
    pub use pixel_mesh::PixelMesh;
    pub use skymesh::SkyMesh;
}

/// Containts render bundle creation methods for screen filters
pub mod filters {
    mod display;

    pub use display::DisplayFilter;
}

/// Contains render bundle creation methods for lights
pub mod lights {
    mod ambient;
    mod sun;

    pub use ambient::AmbientLight;
    pub use sun::SunLight;
}

/// Types related to the render pipeline
pub mod pipeline {
    pub mod ambient;
    pub mod display;
    pub mod gbuffer;
    pub mod mesh;
    pub mod simple;
    pub mod sky_box;
    pub mod sun;
    pub mod vertex3d;

    pub use gbuffer::GBuffer;
    pub use vertex3d::Vertex3D;

    pub(crate) const LIGHT_BLEND: wgpu::BlendState = wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        },
    };
}

pub use context::init;
pub use frame::*;

/// Imports a shader file as a string.
/// In debug mode this will read the shader from a file at runtime
///
/// In release mode this will embed the shader into the binary at build time
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! shader {
    ($path:expr) => {
        std::fs::read_to_string(format!(
            "{}/{}",
            std::path::Path::new(file!()).parent().unwrap().display(),
            $path
        ))
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! shader {
    ($path:expr) => {{
        let a: std::io::Result<_> = Ok(include_str!($path));
        a
    }};
}
