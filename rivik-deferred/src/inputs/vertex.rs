use std::mem;

use crate::types::vertex::Vertex;
use once_cell::sync::Lazy;
use wgpu::vertex_attr_array;

const ATTR_ARRAY: &'static [wgpu::VertexAttribute] = &vertex_attr_array![
    0 => Float32x3, 1 => Float32x3, 2 => Float32x2,
];

pub static LAYOUT: Lazy<wgpu::VertexBufferLayout> = Lazy::new(|| wgpu::VertexBufferLayout {
    array_stride: mem::size_of::<Vertex>() as u64,
    step_mode: wgpu::VertexStepMode::default(),
    attributes: ATTR_ARRAY,
});
