/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! 3d mesh formats

use serde::{Deserialize, Serialize};

mod obj;
pub use obj::*;

/// Vertex type, rexported from mint
pub type Vertex<T> = mint::Point3<T>;
pub type Vertex2<T> = mint::Point2<T>;

/// Common mesh type for importing
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Mesh<T> {
    pub verts: Vec<Vertex<T>>,
    pub normals: Vec<Vertex<T>>,
    pub uvs: Vec<Vertex2<T>>,
}

impl Mesh<f32> {
    pub fn verts(&self) -> VertIter<'_> {
        VertIter {
            index: 0,
            mesh: self,
        }
    }

    pub fn faces(&self) -> FaceIter<'_> {
        FaceIter {
            verts: self.verts(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Vert {
    pub pos: Vertex<f32>,
    pub norm: Option<Vertex<f32>>,
    pub uv: Option<Vertex2<f32>>,
}

pub struct FaceIter<'a> {
    verts: VertIter<'a>,
}

impl<'a> Iterator for FaceIter<'a> {
    type Item = (Vert, Vert, Vert);

    fn next(&mut self) -> Option<Self::Item> {
        Some((self.verts.next()?, self.verts.next()?, self.verts.next()?))
    }
}

pub struct VertIter<'a> {
    index: usize,
    mesh: &'a Mesh<f32>,
}

impl<'a> Iterator for VertIter<'a> {
    type Item = Vert;

    fn next(&mut self) -> Option<Self::Item> {
        let vert = *self.mesh.verts.get(self.index)?;
        let v = Vert {
            pos: vert,
            norm: self.mesh.normals.get(self.index).copied(),
            uv: self.mesh.uvs.get(self.index).copied(),
        };
        self.index += 1;
        Some(v)
    }
}

/// Common scene type for importing
///
/// # Todo
/// - Lights
/// - Emptys
/// - Parenting
/// - Better tagging of nodes
/// - Transforms
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Scene<T> {
    pub nodes: Vec<(Mesh<T>, String)>,
}
