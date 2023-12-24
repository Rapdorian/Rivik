//! Basic version of the engine for testing

use glam::{EulerRot, Quat, Vec3, Vec3A};
use legion::{system, systems, world::SubWorld, Entity, IntoQuery, Query, Resources, World};
use rivik::{
    collision::{
        shapes::{BoundingBox, BoundingSphere},
        BoundingShape, Collider, RigidTransform,
    },
    input::{keys::KeyCode, Button::Key, Input},
    load::load_model_system,
    physics::{self, Kinematic, RigidBody},
    render::{render_system, Transform},
    scene::Scene,
    DeltaTime,
};
use rivik_deferred::types::Model;

// plan:
// For now we'll just implement a simple game with legion and see where it takes us design wise

// I still have no idea how we should hande UI.
// Maybe we'll do something with a global resource

struct Rotate;

#[system(for_each)]
fn rotate(
    kinematic: &mut Kinematic,
    _r: &Rotate,
    #[resource] dt: &DeltaTime,
    #[resource] input: &Input,
) {
    let dt = **dt;
    if input.button(KeyCode::D).is_down() {
        //transform.rotation *= Quat::from_euler(EulerRot::XYZ, 1. * dt, 2. * dt, 3. * dt);
        kinematic.velocity.x += 1.0 * dt;
    }
    if input.button(KeyCode::A).is_down() {
        //transform.rotation *= Quat::from_euler(EulerRot::XYZ, 1. * dt, 2. * dt, 3. * dt);
        kinematic.velocity.x -= 1.0 * dt;
    }
}

struct Demo;

impl Scene for Demo {
    fn update(&mut self, sched: &mut systems::Builder) {
        sched
            .add_system(load_model_system::<String>())
            .add_system(load_model_system::<&'static str>())
            .add_system(rotate_system())
            .add_system(physics::collision_system())
            .add_system(physics::kinematic_system());
    }

    fn draw(&mut self, sched: &mut systems::Builder) {
        sched.add_system(render_system());
    }

    fn init(&mut self, world: &mut World, _resources: &mut Resources) {
        // add sphere to world
        world.push((
            Transform {
                position: Vec3A::ZERO,
                rotation: Quat::default(),
                scale: Vec3A::ONE,
            },
            Model {
                mesh: "rivik-deferred/examples/sphere.mesh",
                diffuse: "rivik-deferred/examples/diffuse.tga",
                rough: "rivik-deferred/examples/specular.tga",
                metal: "rivik-deferred/examples/specular.tga",
                normal: "rivik-deferred/examples/normal.tga",
            },
            BoundingShape::Sphere(BoundingSphere { radius: 1.0 }),
            RigidBody::Static,
        ));

        // add a second object for collision purposes
        world.push((
            Transform {
                position: Vec3A::new(-2.5, 0.0, 0.0),
                rotation: Quat::default(),
                scale: Vec3A::ONE,
            },
            Model {
                mesh: "rivik-deferred/examples/cube.mesh",
                diffuse: "rivik-deferred/examples/diffuse.tga",
                rough: "rivik-deferred/examples/specular.tga",
                metal: "rivik-deferred/examples/specular.tga",
                normal: "rivik-deferred/examples/normal.tga",
            },
            BoundingShape::Box(BoundingBox {
                min: Vec3A::new(-1., -1., -1.),
                max: Vec3A::ONE,
            }),
            Rotate,
            RigidBody::Dynamic { mass: 1.0 },
            Kinematic {
                velocity: Vec3A::ZERO,
            },
        ));
    }
}

fn main() {
    // build the scene
    rivik::start(Demo);
}
