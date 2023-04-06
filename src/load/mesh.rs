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
pub struct GpuMesh<F>(pub F)
where
    F: Format<Output = Mesh<f32>> + Send + Sync,
    F::Error: FormatError + Send + Sync;

impl<F> Format for GpuMesh<F>
where
    F: Format<Output = Mesh<f32>> + Clone + 'static + Send + Sync,
    F::Error: FormatError + Send + Sync,
{
    type Output = CountedBuffer;
    type Error = AssetLoadError;

    fn parse(&self, path: &Path) -> Result<Self::Output, Self::Error> {
        // fetch the asset
        let asset = load(path.to_string(), self.0.clone())?;

        // because we have tri specific information in each vertex there is no reason to index the
        // vertices
        let mut verts: Vec<Vertex3D> = vec![];

        for (a, b, c) in asset.faces() {
            let gen_vert = |main: Vert, a: Vert, b: Vert, c: Vert| {
                let mut vert = Vertex3D::new(main.pos.x, main.pos.y, main.pos.z);
                if let Some(norm) = main.norm {
                    vert = vert.normal(norm.x, norm.y, norm.z);
                }
                if let Some(uv) = main.uv {
                    vert = vert.uv(uv.x, uv.y);
                }

                // add info needed for barycentric coords
                vert = vert.local_space(a.pos.into(), b.pos.into(), c.pos.into());
                if let (Some(a), Some(b), Some(c)) = (a.norm, b.norm, c.norm) {
                    vert = vert.tri_norm(a.into(), b.into(), c.into());
                }
                if let (Some(a), Some(b), Some(c)) = (a.uv, b.uv, c.uv) {
                    vert = vert.tri_uv(a.into(), b.into(), c.into());
                }
                vert
            };

            verts.push(gen_vert(a, a, b, c));
            verts.push(gen_vert(b, a, b, c));
            verts.push(gen_vert(c, a, b, c));
        }

        // create buffer out of vertex list
        let mut buffer = vec![];
        for v in &verts {
            buffer.extend_from_slice(bytemuck::bytes_of(v));
        }

        // upload buffer to GPU
        Ok(CountedBuffer::new(
            device().create_buffer_init(&BufferInitDescriptor {
                label: Some(&path.to_string()),
                contents: &buffer,
                usage: BufferUsages::VERTEX,
            }),
            verts.len() as u32,
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
