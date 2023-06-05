use assets::{formats::mesh::ObjMesh, load};
use image::ImageFormat;
use pollster::block_on;
use rivik_render::{
    context::surface_config,
    draw::{mesh::vertex_buffer, Bundle, Mesh},
    jobs::deferred::Deferred,
    lights::SunLight,
    load::{GpuMesh, GpuTexture},
    render_job::RenderJob,
    transform::Spatial,
    Frame,
};
use ultraviolet::Vec3;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

fn main() {
    // create a window
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

    // init engine
    block_on(rivik_render::init(&window, (1920, 1080))).unwrap();

    // load a mesh
    let mesh = {
        let mesh = load(
            "file:assets/fighter_smooth.obj",
            GpuMesh(ObjMesh, vertex_buffer),
        )
        .unwrap();

        let tex = load(
            "file:assets/fighter.albedo.png",
            GpuTexture(ImageFormat::Png),
        )
        .unwrap();
        Mesh::new(mesh, tex)
    };

    let light = SunLight::new([1.0, 1.0, 1.0], [-1.0, 1.0, 0.0]);

    // setup a render pipeline
    let mut pipeline = Deferred::default();

    // add objects into this pipeline
    pipeline.push_geom(&mesh);
    pipeline.push_light(light.bundle());

    // update camera matrix
    let aspect = {
        let config = surface_config().read().unwrap();
        config.width as f32 / config.height as f32
    };

    let eye = Vec3::new(2.0, 2.0, 2.0);
    let focus = Vec3::new(0.0, 0.0, 0.0);
    let proj = ultraviolet::projection::perspective_vk(80.0, aspect, 0.1, 100.0);
    let view = ultraviolet::Mat4::look_at(eye, focus, Vec3::unit_y());

    light.transform().update_view(proj, view);
    mesh.transform().update_view(proj, view);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = ControlFlow::Exit,
        Event::RedrawRequested(..) => {
            // request a frame
            let mut frame = Frame::new().unwrap();

            //  ```
            //  pbr_pass(|r| {
            //      basic_mesh(|r| {
            //          r.draw(mesha);
            //          r.draw(meshb);
            //      });
            //
            //      pixel_mesh(|r| {
            //          r.draw(alt_material);
            //      });
            //
            //      r.light(ambient);
            //      r.light(sun);
            //  });
            //
            //  special_pass(|r| {
            //      r.draw(special_material);
            //      r.light(ambient);
            //      r.light(sun);
            //  });
            //
            //  ui_pass(|| { /* ui commands */);
            //  ```

            // run a render pipeline
            pipeline.run(&mut frame);

            // finish frame
            frame.present();
        }
        _ => {}
    });
}
