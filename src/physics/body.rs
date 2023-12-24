use glam::Vec3A;
use legion::{system, world::SubWorld, IntoQuery};

use crate::{
    collision::{BoundingShape, Collider},
    physics::Kinematic,
    render::Transform,
};

#[derive(Clone, Copy, Debug)]
pub enum RigidBody {
    Dynamic { mass: f32 },
    Static,
}

#[system]
#[read_component(Transform)]
#[read_component(BoundingShape)]
#[write_component(Kinematic)]
#[read_component(RigidBody)]
pub fn collision(world: &mut SubWorld) {
    let mut colliders: Vec<_> = <(
        &Transform,
        &BoundingShape,
        &RigidBody,
        Option<&mut Kinematic>,
    )>::query()
    .iter_mut(world)
    .collect();
    // find collisions
    for a in 0..colliders.len() {
        // skip duplicate entryies
        for b in a + 1..colliders.len() {
            // check entities
            if Collider::new(*colliders[a].0, *colliders[a].1)
                .check_collision(&Collider::new(*colliders[b].0, *colliders[b].1))
            {
                // found a collision if either object is a dynamic body then clear its kinematic
                if let RigidBody::Dynamic { .. } = colliders[a].2 {
                    if let Some(k) = &mut colliders[a].3 {
                        k.velocity = Vec3A::ZERO;
                    }
                }
                if let RigidBody::Dynamic { .. } = colliders[b].2 {
                    if let Some(k) = &mut colliders[b].3 {
                        k.velocity = Vec3A::ZERO;
                    }
                }
            }
        }
    }
}

#[system(for_each)]
pub fn kinematic(t: &mut Transform, k: &Kinematic) {
    t.position += k.velocity
}
