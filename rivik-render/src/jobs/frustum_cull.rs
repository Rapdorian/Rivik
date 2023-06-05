/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! Implementation of Frustum Culling

use std::{
    cmp::Ordering,
    sync::{Arc, RwLock},
};

use glam::{Mat4, Vec3A};
use mint::{ColumnMatrix4, Vector3};

/// Render storage that provides an interface for querying objects within a frustum
pub struct FrustumCulled<R> {
    data: Vec<(R, Arc<RwLock<Mat4>>, AABB)>,
}

impl<R> Default for FrustumCulled<R> {
    fn default() -> Self {
        Self { data: Vec::new() }
    }
}

impl<R> FrustumCulled<R> {
    /// Insert a renderable object into this frustum structure
    pub fn push(&mut self, renderable: R, transform: Arc<RwLock<Mat4>>, bounds: AABB) {
        self.data.push((renderable, transform, bounds));
    }

    /// Returns an interator over the renderables present inside the render frustum
    pub fn cull(&self, f: Frustum) -> impl Iterator<Item = &R> {
        FrustumCullIter {
            data: &self.data,
            f,
        }
    }
}

#[allow(missing_docs)]
pub struct FrustumCullIter<'a, R> {
    data: &'a [(R, Arc<RwLock<Mat4>>, AABB)],
    f: Frustum,
}

impl<'a, R> Iterator for FrustumCullIter<'a, R> {
    type Item = &'a R;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let d = self.data.first();

            if let Some(d) = d {
                self.data = &self.data[1..];
                // check that d is within the frustum
                if self.f.contains(&d.2) {
                    return Some(&d.0);
                }
            } else {
                return None;
            }
        }
    }
}

#[derive(Debug)]
struct Plane {
    n: Vec3A,
    d: f32,
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Frustum {
    left: Plane,
    right: Plane,
    bottom: Plane,
    top: Plane,
    near: Plane,
    far: Plane,
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Default, Debug)]
pub struct AABB {
    min: Vec3A,
    max: Vec3A,
}

impl AABB {
    /// Set the minimum point of this box
    pub fn min(&mut self, min: impl Into<Vector3<f32>>) {
        self.min = Vec3A::from(min.into());
    }

    /// Set the maximum point of this box
    pub fn max(&mut self, max: impl Into<Vector3<f32>>) {
        self.max = Vec3A::from(max.into());
    }

    fn verts(&self) -> [Vec3A; 8] {
        [
            Vec3A::new(self.min.x, self.min.y, self.min.z),
            Vec3A::new(self.max.x, self.min.y, self.min.z),
            Vec3A::new(self.min.x, self.max.y, self.min.z),
            Vec3A::new(self.max.x, self.max.y, self.min.z),
            Vec3A::new(self.min.x, self.min.y, self.max.z),
            Vec3A::new(self.max.x, self.min.y, self.max.z),
            Vec3A::new(self.min.x, self.max.y, self.max.z),
            Vec3A::new(self.max.x, self.max.y, self.max.z),
        ]
    }
}

impl Plane {
    /// Create a new plane
    pub fn new(n: Vec3A, d: f32) -> Self {
        let l = n.length();

        Self { n: n / l, d: d / l }
    }

    fn cmp(&self, b: &AABB) -> Ordering {
        // ALG: project each vertex of the AABB onto the planes normal vector
        // the projection can be compared with d to determine which side of the plane each vertex
        // is on

        /// Test a single vertex to determine which side of the plane it is on
        fn test(p: &Plane, v: &Vec3A) -> Ordering {
            let proj = (v.project_onto(p.n).length()) - p.d;
            proj.total_cmp(&0.0)
        }

        let verts = b.verts();
        let mut verts = verts.iter();

        let mut order = test(self, verts.next().unwrap());

        // consume the rest of the verts checking the order
        for v in verts {
            // early return if we hit an intersection
            if order.is_eq() {
                return order;
            }

            // If two verts are on opposite sides of the plane that is an intersection
            if order != test(self, v) {
                return Ordering::Equal;
            }
        }
        order
    }
}

impl Frustum {
    /// Create a frustum from a transform
    pub fn new(transform: impl Into<ColumnMatrix4<f32>>) -> Frustum {
        let t = Mat4::from(transform.into());

        let left: Vec<f32> = (0..4).map(|i| t.col(i)[3] + t.col(i)[0]).collect();
        let right: Vec<f32> = (0..4).map(|i| t.col(i)[3] - t.col(i)[0]).collect();
        let bottom: Vec<f32> = (0..4).map(|i| t.col(i)[3] + t.col(i)[1]).collect();
        let top: Vec<f32> = (0..4).map(|i| t.col(i)[3] - t.col(i)[1]).collect();
        let near: Vec<f32> = (0..4).map(|i| t.col(i)[3] + t.col(i)[2]).collect();
        let far: Vec<f32> = (0..4).map(|i| t.col(i)[3] - t.col(i)[2]).collect();

        dbg!(&near);
        dbg!(&far);

        // create frustum
        Self {
            left: Plane::new(Vec3A::new(left[0], left[1], left[2]), left[3]),
            right: Plane::new(Vec3A::new(right[0], right[1], right[2]), right[3]),
            bottom: Plane::new(Vec3A::new(bottom[0], bottom[1], bottom[2]), bottom[3]),
            top: Plane::new(Vec3A::new(top[0], top[1], top[2]), top[3]),
            near: Plane::new(Vec3A::new(near[0], near[1], near[2]), near[3]),
            far: Plane::new(Vec3A::new(far[0], far[1], far[2]), far[3]),
        }
    }

    fn contains(&self, b: &AABB) -> bool {
        // ALG: Check each plane of the frustum in a reasonable order. If the box lies fully
        // outside any plane then we can early return with false

        if self.near.cmp(b).is_gt() {
            return false;
        }
        if self.left.cmp(b).is_gt() {
            return false;
        }
        if self.right.cmp(b).is_gt() {
            return false;
        }
        if self.top.cmp(b).is_gt() {
            return false;
        }
        if self.bottom.cmp(b).is_gt() {
            return false;
        }
        if self.far.cmp(b).is_gt() {
            return false;
        }
        true
    }
}
