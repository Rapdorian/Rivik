//! Loads a mesh file into the GPU

use std::ops::Deref;

use assets::{
    formats::{
        mesh::{Mesh, Vert},
        FormatError,
    },
    load, AssetLoadError, Format, Path,
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages,
};

use crate::{context::device, pipeline::Vertex3D};

/// Import format for a Mesh
///
/// will add any metadata needed for the renderer to operate
pub struct GpuMesh<F, V>(pub F, pub V)
where
    F: Format<Output = Mesh<f32>> + Send + Sync,
    F::Error: FormatError + Send + Sync,
    V: Fn(&Mesh<f32>) -> (Vec<u8>, usize);

impl<F, V> Format for GpuMesh<F, V>
where
    F: Format<Output = Mesh<f32>> + Clone + 'static + Send + Sync,
    F::Error: FormatError + Send + Sync,
    V: Fn(&Mesh<f32>) -> (Vec<u8>, usize),
{
    type Output = CountedBuffer;
    type Error = AssetLoadError;

    fn parse(&self, path: &Path) -> Result<Self::Output, Self::Error> {
        // fetch the asset
        let asset = load(path.to_string(), self.0.clone())?;

        let (buffer, len) = (self.1)(&asset);

        // upload buffer to GPU
        Ok(CountedBuffer::new(
            device().create_buffer_init(&BufferInitDescriptor {
                label: Some(&path.to_string()),
                contents: &buffer,
                usage: BufferUsages::VERTEX,
            }),
            len as u32,
        ))
    }
}

pub struct CountedBuffer {
    len: u32,
    buffer: Buffer,
}

impl CountedBuffer {
    pub fn new(buf: Buffer, len: u32) -> Self {
        Self { len, buffer: buf }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> u32 {
        self.len
    }
}

impl Deref for CountedBuffer {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
