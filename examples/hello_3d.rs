/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

use glam::Vec3;
use rivik::{run, Handle, Report};
use rivik_assets::{
    formats::{img::ImageFormat, mesh::ObjMesh},
    load,
};
use rivik_render::{
    draw::{pixel_mesh, PixelMesh},
    lights::SunLight,
    load::{GpuMesh, GpuTexture},
    tracing::UiSubscriber,
    Transform,
};

use tracing::{dispatcher::set_global_default, Dispatch};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, Registry};

trait PrettyUnwrap {
    type OUT;
    fn pretty_unwrap(self) -> Self::OUT;
}

pub struct App {
    mesh: Handle<PixelMesh>,
}

impl rivik::App for App {
    fn init(scene: &mut rivik::Context) -> Result<Self, Report> {
        // load a model
        let mesh = load(
            "file:assets/fighter_smooth.obj",
            GpuMesh(ObjMesh, pixel_mesh::vertex_buffer),
        )?;

        let tex = load("file:assets/fighter.png", GpuTexture(ImageFormat::Png))?;

        let mesh = PixelMesh::new(mesh, Transform::default(), tex);
        scene.insert_light(SunLight::new(
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(-1., 1., 0.),
        ));

        Ok(Self {
            mesh: scene.insert(mesh),
        })
    }
}

fn main() {
    set_global_default(Dispatch::new(
        Registry::default().with(UiSubscriber::default()),
    ))
    .unwrap();
    run::<App>();
}
