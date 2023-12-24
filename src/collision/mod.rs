//! Types for handling collision
// This module needs to be reworked:
// we need to seperate out collisions logically into broad and narrow phase collision.
// narrow phase collisions will use the seperating axis theorem to allow arbitrary convex meshes.
// broad phase collision will be AABBs and points.
// At some level their may also be a clustering algorithm to speed up collision checking but that
// is outside the scope of this module

use std::ops::Mul;

use glam::{Affine3A, Vec3A};

mod sat;

pub mod shapes {
    mod aabb;
    mod bbox;
    mod sphere;

    pub use aabb::*;
    pub use bbox::*;
    pub use sphere::*;
}
use shapes::*;

use crate::render::Transform;

use self::sat::SATShape;

#[derive(Copy, Clone, Debug)]
pub enum BoundingShape {
    Box(BoundingBox),
    Sphere(BoundingSphere),
}

#[derive(Copy, Clone, Debug)]
pub struct Collider {
    pub transform: Affine3A,
    pub shape: BoundingShape,
}

impl Collider {
    pub fn new(transform: Transform, shape: BoundingShape) -> Self {
        Self {
            transform: Affine3A::from_scale_rotation_translation(
                transform.scale.into(),
                transform.rotation,
                transform.position.into(),
            ),
            shape,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RigidTransform {
    pub pos: Vec3A,
    pub rot: glam::Quat,
}

impl Mul<Vec3A> for RigidTransform {
    type Output = Vec3A;

    fn mul(self, rhs: Vec3A) -> Self::Output {
        (self.rot * rhs) + self.pos
    }
}

impl Collider {
    pub fn check_collision(&self, other: &Collider) -> bool {
        // collect axes
        for axis in self.axes() {
            // check for a gap in each axis
            let a = self.project(axis);
            let b = other.project(axis);
            if a.end < b.start || b.end < a.start {
                // Found a gap
                return false;
            }
        }

        for axis in other.axes() {
            // check for a gap in each axis
            let a = self.project(axis);
            let b = other.project(axis);
            if a.end < b.start || b.end < a.start {
                // Found a gap
                return false;
            }
        }

        true
    }
}
