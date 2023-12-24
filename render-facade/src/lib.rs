//! Render facade for rivik
//!
//! Provides a set of type for describing a render scene

// I want to be able to expirement with deferred/forward+ with the same higher level code

// Which means I can't expose render structure in this API

// I am willing to make the following assumptions for sanity reasons
// - Vertex Format

pub struct Mesh {
    verts: wgpu::Buffer,
    v_len: u16,
    index: wgpu::Buffer,
    i_len: u16,
    mat: u16,
}

pub struct Material {
    albedo: wgpu::Buffer,
    specular: wgpu::Buffer,
    hardness: wgpu::Buffer,
}

pub struct AABB {
    min: [f32; 3],
    max: [f32; 3],
}

pub struct Model {
    mesh: Vec<Mesh>,
    mat: Vec<Material>,
    bound: AABB,
}

pub struct SceneNode {
    pos: [f32; 3],
    node: Mesh,
}

pub enum Light {
    Ambient { color: [f32; 3] },
    Sun { color: [f32; 3], dir: [f32; 3] },
    Point { color: [f32; 3], pos: [f32; 3] },
}

pub struct Scene {
    geom: Vec<SceneNode>,
    lights: Vec<Light>,
}
