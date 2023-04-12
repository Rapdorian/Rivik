#![allow(missing_docs)]

///! A simple 3d vertex
use bytemuck::{Pod, Zeroable};
use ultraviolet::{Vec2, Vec3};
use wgpu_macros::VertexLayout;

/// 3d Vertex
///
/// TODO: I'd like to try to optimize this into something smaller later but \_(ツ)_/
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod, VertexLayout, Default)]
pub struct Vertex3D {
    pos: [f32; 3],
    norm: [f32; 3],
    uv: [f32; 2],
    uv_a: [f32; 2],
    uv_b: [f32; 2],
    uv_c: [f32; 2],
    pos_a: [f32; 3],
    pos_b: [f32; 3],
    pos_c: [f32; 3],
    norm_a: [f32; 3],
    norm_b: [f32; 3],
    norm_c: [f32; 3],
}

impl Vertex3D {
    /// Create a new 3d pixel vertex
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            pos: [x, y, z],
            norm: *Vec3::new(1.0, 1.0, 1.0).normalized().as_array(),
            ..Self::default()
        }
    }

    /// Add neighboring uvs to this vertex
    pub fn tri_uv(self, uv_a: Vec2, uv_b: Vec2, uv_c: Vec2) -> Self {
        Self {
            uv_a: *uv_a.as_array(),
            uv_b: *uv_b.as_array(),
            uv_c: *uv_c.as_array(),
            ..self
        }
    }

    /// Add neighboring normals to this vertex
    pub fn tri_norm(self, norm_a: Vec3, norm_b: Vec3, norm_c: Vec3) -> Self {
        Self {
            norm_a: *norm_a.as_array(),
            norm_b: *norm_b.as_array(),
            norm_c: *norm_c.as_array(),
            ..self
        }
    }

    /// Add neighboring positions this vertex
    pub fn local_space(self, pos_a: Vec3, pos_b: Vec3, pos_c: Vec3) -> Self {
        Self {
            pos_a: *pos_a.as_array(),
            pos_b: *pos_b.as_array(),
            pos_c: *pos_c.as_array(),
            ..self
        }
    }

    /// Add uv coordinates to this vertex
    pub fn uv(self, u: f32, v: f32) -> Self {
        Self { uv: [u, v], ..self }
    }

    /// Add a normal to this vertex
    pub fn normal(self, x: f32, y: f32, z: f32) -> Self {
        Self {
            norm: [x, y, z],
            ..self
        }
    }
}
