//! Basic deferred renderer for rivik engine

use std::{marker::PhantomData, num::NonZeroU32};

use context::{device, queue};
use glam::Mat4;
use pollster::block_on;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

pub use rivik_render_types as types;

pub mod context;
pub mod frame;
pub mod gbuffer;

pub mod pipes {
    pub mod ambient;
    pub mod mesh;
    pub mod sun;

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

pub mod inputs {
    pub mod material;
    pub mod sampler;
    pub mod sun_light;
    pub mod transform;
    pub mod vertex;

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
            .map(|shader| {
                $crate::context::device().create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some($path),
                    source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&shader)),
                })
            })
        };
    }

    #[cfg(not(debug_assertions))]
    #[allow(missing_docs)]
    #[macro_export]
    macro_rules! shader {
        ($path:expr) => {{
            let a: std::io::Result<_> = Ok(include_str!($path));
            a.map(|shader| {
                $crate::context::device().create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some($path),
                    source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&shader)),
                })
            })
        }};
    }
}

/// Entry point to the renderer
#[derive(Default)]
pub struct Render {
    vertex_buffers: Vec<wgpu::Buffer>,
    index_buffers: Vec<(wgpu::Buffer, usize)>,
    textures: Vec<(wgpu::Texture, wgpu::TextureView)>,
}

impl Render {
    pub fn new<W>(w: &W) -> Self
    where
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        block_on(context::init(w));
        Render::default()
    }
}

impl<'a> types::Renderer<'a> for Render {
    type FRAME = frame::Frame<'a>;
    type ID = u32;

    fn frame(&'a mut self) -> Self::FRAME {
        frame::Frame::new(&self.vertex_buffers, &self.index_buffers, &self.textures)
            .expect("Failed to acquire a frame")
    }

    fn upload_mesh<M: types::mesh::AsMesh>(&mut self, label: Option<&str>, mesh: &M) -> Self::ID {
        // upload a mesh

        let v_buffer = device().create_buffer_init(&BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(&mesh.verts()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let i_buffer = device().create_buffer_init(&BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(&mesh.faces()),
            usage: wgpu::BufferUsages::INDEX,
        });

        assert_eq!(self.vertex_buffers.len(), self.index_buffers.len());
        let id = self.vertex_buffers.len();

        self.vertex_buffers.push(v_buffer);
        self.index_buffers.push((i_buffer, mesh.faces().len() * 3));
        id as u32
    }

    fn upload_texture<T: types::tex::AsTex>(&mut self, label: Option<&str>, tex: &T) -> Self::ID {
        // upload this texture to the GPU
        let texture_size = wgpu::Extent3d {
            width: tex.width() as u32,
            height: tex.height() as u32,
            depth_or_array_layers: 1,
        };

        // determine Texture Format from width
        let pixel_size = tex.texel_width();
        let format = match pixel_size {
            4 => wgpu::TextureFormat::Rgba8UnormSrgb,
            size => panic!("Invalid pixel type: {size}"),
        };

        let texture = device().create_texture(&wgpu::TextureDescriptor {
            label,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: Default::default(),
        });

        let desc = wgpu::TextureViewDescriptor {
            label,
            format: Some(format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(1),
        };

        let view = texture.create_view(&desc);

        // write image into texture
        queue().write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            tex.buffer(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some((pixel_size * tex.width()) as u32),
                rows_per_image: Some(tex.height() as u32),
            },
            texture_size,
        );

        let id = self.textures.len();
        self.textures.push((texture, view));
        id as u32
    }
}
