use std::f32::consts::PI;

use assets::{formats::mesh::ObjMesh, load};
use image::ImageFormat;
use log::{error, info};
use pollster::block_on;
use reerror::{conversions::internal, throw, Result};
use rivik_render::{
    context::{device, resize, surface_config},
    draw::mesh::MeshRenderable,
    filters::display::DisplayFilter,
    lights::{ambient::AmbientLight, sun::SunLight},
    load::{GpuMesh, GpuTexture},
    Frame, Transform,
};
use ultraviolet::{Mat4, Vec3, Vec4};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BufferUsages,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

async fn run() -> Result<()> {
    // init window
    let event_loop = EventLoop::new();
    env_logger::init();
    let window = Window::new(&event_loop).map_err(internal)?;
    rivik_render::init(&window, (1920, 1080)).await;

    // load a mesh
    let mesh = throw!(load("file:assets/fighter_smooth.obj", GpuMesh(ObjMesh)));
    let tex = load(
        "file:assets/fighter.albedo.png",
        GpuTexture(ImageFormat::Png),
    )?;

    let aspect = {
        let config = surface_config().read()?;
        config.width as f32 / config.height as f32
    };

    let eye = Vec3::new(2.0, 2.0, 2.0);
    let focus = Vec3::new(0.0, 0.0, 0.0);

    let mut proj = ultraviolet::projection::perspective_vk(80.0, aspect, 0.1, 100.0);
    let mut view = ultraviolet::Mat4::look_at(eye, focus, Vec3::unit_y());

    let mut model = ultraviolet::Mat4::identity();

    let transform = Transform::new(proj, view, model);
    let mesh_bundle = MeshRenderable::new(mesh, &transform, tex)?;

    let mut i = 0;

    let ambient = AmbientLight::new(0.01, 0.01, 0.01);
    let sun = SunLight::new(Vec3::new(1.0, 1.0, 1.0), Vec3::new(-1., 1., 0.));

    let display = DisplayFilter::default();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                resize(size.width, size.height);

                // re-compute projection matrix
                let aspect = {
                    let config = surface_config().read().unwrap();
                    config.width as f32 / config.height as f32
                };

                proj = ultraviolet::projection::perspective_vk(80.0, aspect, 0.1, 100.0);
            }
            Event::RedrawRequested(..) => {
                i = (i + 1) % 4096;

                model = Mat4::from_rotation_y(i as f32 / (4096.0 / (2. * PI)))
                    * Mat4::from_translation(Vec3::new(0.0, -0.7, 0.0));
                transform.update(proj, view, model);

                let mut frame = Frame::new().unwrap();

                frame.draw_geom(&mesh_bundle);
                frame.draw_light(&sun);
                frame.draw_light(&ambient);
                frame.draw_filter(&display);

                frame.present();
                window.request_redraw();
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    })
}

fn main() {
    if let Err(e) = block_on(run()) {
        error!("{}", e);
    }
}
