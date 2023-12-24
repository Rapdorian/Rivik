use glam::{Mat4, Quat, Vec3, Vec3A};
use legion::{system, world::SubWorld, Query};
use rivik_deferred::{
    types::{light::Light, Frame, Model, Renderer},
    Render,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3A,
    pub rotation: Quat,
    pub scale: Vec3A,
}

#[system]
pub fn render(
    #[resource] render: &mut Render,
    world: &mut SubWorld,
    query: &mut Query<(&Transform, &Model<u32>)>,
) {
    // create a frame and queue everything
    // This may not be the best way to do this.
    // Creating a frame resource that is updated per frame may be better
    let mut frame = render.frame();
    //frame.draw_light(Light::Directional {
    //    color: [1.0; 3],
    //    direction: [-1., 0., 1.],
    //});

    frame.draw_light(Light::Directional {
        color: [1.0; 3],
        direction: [1., 0., 0.],
    });

    frame.draw_light(Light::Directional {
        color: [1.0; 3],
        direction: [-1., 0., 0.],
    });

    let cam = Mat4::look_at_rh(Vec3::new(-10.0, -30.0, 10.0), Vec3::ZERO, Vec3::Z);
    frame.set_camera(cam);
    for (transform, model) in query.iter(world) {
        // only render loaded models
        frame.draw_mesh(
            model,
            transform.position,
            transform.rotation,
            transform.scale,
        );
    }
}
