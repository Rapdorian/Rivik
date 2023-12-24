//! Seperating Axis Theorem helpers

use std::ops::Range;

use glam::Vec3A;

use super::Collider;

pub trait SATShape {
    fn axes(&self) -> Vec<Vec3A>;
    fn project(&self, axis: Vec3A) -> Range<f32>;
}

impl SATShape for Collider {
    fn axes(&self) -> Vec<Vec3A> {
        match self.shape {
            super::BoundingShape::Box(_) => {
                vec![
                    self.transform.transform_vector3a(Vec3A::X).normalize(),
                    self.transform.transform_vector3a(Vec3A::Y).normalize(),
                    self.transform.transform_vector3a(Vec3A::Z).normalize(),
                ]
            }
            super::BoundingShape::Sphere(_) => {
                vec![Vec3A::X, Vec3A::Y, Vec3A::Z]
            }
        }
    }

    fn project(&self, axis: Vec3A) -> Range<f32> {
        assert!(axis.is_normalized());
        match self.shape {
            super::BoundingShape::Box(shape) => {
                let v = [shape.min, shape.max];

                let mut proj = (f32::INFINITY, f32::NEG_INFINITY);

                for a in 0..2 {
                    for b in 0..2 {
                        for c in 0..2 {
                            // generate a point
                            let p = self
                                .transform
                                .transform_point3a(Vec3A::new(v[a].x, v[b].y, v[c].z));

                            // record min/max projection of point onto axis
                            let p = p.project_onto_normalized(axis).length();
                            if p < proj.0 {
                                proj.0 = p;
                            }
                            if p > proj.1 {
                                proj.1 = p;
                            }
                        }
                    }
                }
                proj.0..proj.1
            }
            super::BoundingShape::Sphere(shape) => {
                // project center onto axis
                let center = self.transform.transform_point3a(Vec3A::ZERO);
                let p = center.project_onto_normalized(axis).length();
                p - shape.radius..p + shape.radius
            }
        }
    }
}
