/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! Loads a mesh file into the GPU

use std::ops::Deref;

use assets::{
    formats::{mesh::Mesh, FormatError},
    load, AssetLoadError, Format, Path,
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages,
};

use crate::{context::device, jobs::frustum_cull::AABB};

/// Import format for a Mesh
///
/// will add any metadata needed for the renderer to operate
pub struct GpuMesh<F, V>(pub F, pub V)
where
    F: Format<Output = Mesh<f32>> + Send + Sync,
    F::Error: FormatError + Send + Sync,
    V: Fn(&Mesh<f32>) -> (Vec<u8>, usize, AABB);

impl<F, V> Format for GpuMesh<F, V>
where
    F: Format<Output = Mesh<f32>> + Clone + 'static + Send + Sync,
    F::Error: FormatError + Send + Sync,
    V: Fn(&Mesh<f32>) -> (Vec<u8>, usize, AABB),
{
    type Output = CountedBuffer;
    type Error = AssetLoadError;

    fn parse(&self, path: &Path) -> Result<Self::Output, Self::Error> {
        // fetch the asset
        let asset = load(path.to_string(), self.0.clone())?;
        println!("Fetching asset: {path}");

        let (buffer, len, aabb) = (self.1)(&asset);

        // upload buffer to GPU
        Ok(CountedBuffer::new(
            device().create_buffer_init(&BufferInitDescriptor {
                label: Some(&path.to_string()),
                contents: &buffer,
                usage: BufferUsages::VERTEX,
            }),
            len as u32,
            aabb,
        ))
    }
}

/// A GPU buffer with a length
pub struct CountedBuffer {
    len: u32,
    buffer: Buffer,
    aabb: AABB,
}

impl CountedBuffer {
    /// Creates a new `CountedBuffer`
    pub fn new(buf: Buffer, len: u32, aabb: AABB) -> Self {
        Self {
            len,
            buffer: buf,
            aabb,
        }
    }

    /// Get the length of this buffer
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> u32 {
        self.len
    }

    /// Gets an axis-aligned bounding box for this mesh
    pub fn bounds(&self) -> AABB {
        self.aabb
    }
}

impl Deref for CountedBuffer {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl Drop for CountedBuffer {
    fn drop(&mut self) {
        println!("Dropping a GPU vertex buffer");
    }
}
