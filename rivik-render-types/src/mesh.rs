use bytemuck::{Pod, Zeroable};

use crate::vertex::Vertex;

/// Simple mesh data
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Mesh<const V: usize, const F: usize> {
    pub verts: [Vertex; V],
    pub faces: [[u16; 3]; F],
}

unsafe impl<const V: usize, const F: usize> Pod for Mesh<V, F> {}
unsafe impl<const V: usize, const F: usize> Zeroable for Mesh<V, F> {}

#[repr(C)]
pub struct MeshPtr<'a> {
    data: &'a [u8],
}

impl<'a> MeshPtr<'a> {
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    fn v_len(&self) -> usize {
        u16::from_le_bytes([self.data[0], self.data[1]]) as usize
    }

    // fn i_len(&self) -> usize {
    //     u16::from_le_bytes([self.data[2], self.data[3]]) as usize
    // }

    fn verts(&self) -> &[Vertex] {
        bytemuck::cast_slice(&self.data[4..4 + self.v_len()])
    }

    fn faces(&self) -> &[[u16; 3]] {
        bytemuck::cast_slice(&self.data[4 + self.v_len()..])
    }
}

impl<'a> From<&'a [u8]> for MeshPtr<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self { data: value }
    }
}

impl<'a> AsMesh for MeshPtr<'a> {
    fn verts(&self) -> &[Vertex] {
        self.verts()
    }

    fn faces(&self) -> &[[u16; 3]] {
        self.faces()
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct DynMesh {
    pub verts: Vec<Vertex>,
    pub faces: Vec<[u16; 3]>,
}

impl From<&[u8]> for DynMesh {
    fn from(value: &[u8]) -> Self {
        // get sizes
        let size_buffer = &value[0..4];
        let sizes: &[u16] = bytemuck::cast_slice(size_buffer);
        let i_buf_offset = sizes[0] as usize + 4;
        let v_buffer = &value[4..i_buf_offset];
        let i_buffer = &value[i_buf_offset..];

        let v_buffer: &[Vertex] = bytemuck::cast_slice(v_buffer);
        let i_buffer: &[[u16; 3]] = bytemuck::cast_slice(i_buffer);
        Self {
            verts: v_buffer.to_vec(),
            faces: i_buffer.to_vec(),
        }
    }
}

pub trait AsMesh {
    fn verts(&self) -> &[Vertex];
    fn faces(&self) -> &[[u16; 3]];
}

impl<const V: usize, const F: usize> AsMesh for Mesh<V, F> {
    fn verts(&self) -> &[Vertex] {
        &self.verts
    }

    fn faces(&self) -> &[[u16; 3]] {
        &self.faces
    }
}

impl AsMesh for DynMesh {
    fn verts(&self) -> &[Vertex] {
        &self.verts
    }

    fn faces(&self) -> &[[u16; 3]] {
        &self.faces
    }
}
