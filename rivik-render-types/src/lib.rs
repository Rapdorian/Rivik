//! Set of types for describing a 3d scene
//!
//! All types are intended to be able to be directly copied into the GPU
//!
//! TODO: More complicated mesh types.
//! For example, we need to be able to tag meshes with extra data. If we want to do static
//! reflections we'll need to tag some meshes as static-reflection so the renderable can filter
//! that pass.
//! There are probably other things that would be useful to tag a mesh with (shadows casting?).

use egui::{ClippedPrimitive, TexturesDelta};
use material::Material;
use mesh::AsMesh;

pub mod light;
pub mod material;
pub mod mesh;
pub mod tex;
pub mod vertex;

pub use egui;
use mint::{ColumnMatrix4, Quaternion, Vector3};
use serde::{Deserialize, Serialize};

#[cfg(test)]
pub mod mock;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Model<ID> {
    pub mesh: ID,
    pub diffuse: ID,
    pub rough: ID,
    pub metal: ID,
    pub normal: ID,
}

/// Interface for building a frame.
pub trait Frame<'f> {
    type ID: Clone;

    fn draw_mesh(
        &mut self,
        model: &'f Model<Self::ID>,
        pos: impl Into<Vector3<f32>>,
        rot: impl Into<Quaternion<f32>>,
        scale: impl Into<Vector3<f32>>,
    );
    fn draw_light(&mut self, light: light::Light);
    fn set_camera(&mut self, cam: impl Into<ColumnMatrix4<f32>>);
    fn set_ui<'a>(&'a mut self, clip_prim: &'a [ClippedPrimitive], textures: TexturesDelta);
}

/// Describe a handle to a renderer
pub trait Renderer<'f> {
    type FRAME: Frame<'f, ID = Self::ID>;
    type ID;

    fn frame(&'f mut self) -> Self::FRAME;

    fn upload_mesh<M: mesh::AsMesh>(&mut self, label: Option<&str>, mesh: &M) -> Self::ID;
    fn upload_texture<T: tex::AsTex>(&mut self, label: Option<&str>, tex: &T) -> Self::ID;
}
