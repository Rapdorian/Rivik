use std::borrow::Borrow;

use glam::Vec3;
use rivik::{run, Handle};
use rivik_assets::{
    formats::{img::ImageFormat, mesh::ObjMesh},
    load,
};
use rivik_render::{
    draw::{pixel_mesh, PixelMesh},
    lights::sun::SunLight,
    load::{GpuMesh, GpuTexture},
    tracing::UiSubscriber,
    Transform,
};
use tracing::{dispatcher::set_global_default, Dispatch};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, Registry};

pub struct App {
    mesh: Handle<PixelMesh>,
}

impl rivik::App for App {
    fn init(scene: &mut rivik::Context) -> Self {
        // load a model
        let mesh = load(
            "file:../render/assets/fighter_smooth.obj",
            GpuMesh(ObjMesh, pixel_mesh::vertex_buffer),
        )
        .unwrap();
        let tex = load(
            "file:../render/assets/fighter.albedo.png",
            GpuTexture(ImageFormat::Png),
        )
        .unwrap();

        let mesh = PixelMesh::new(mesh, Transform::default(), tex);
        scene.insert_light(SunLight::new(
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(-1., 1., 0.),
        ));

        Self {
            mesh: scene.insert(mesh),
        }
    }
}

fn main() {
    set_global_default(Dispatch::new(
        Registry::default().with(UiSubscriber::default()),
    ))
    .unwrap();
    run::<App>();
}
