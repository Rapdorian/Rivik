use std::time::Duration;

use glam::{Mat4, Quat, Vec3};
use include_bytes_aligned::include_bytes_aligned;
use rivik_deferred::{context, Render};
use rivik_render_types::{
    light::Light,
    material::Material,
    mesh::{DynMesh, Mesh, MeshPtr},
    tex::DynTex,
    vertex::Vertex,
    Frame, Model, Renderer,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

fn load_tga(rend: &mut Render, tex: &[u8]) -> u32 {
    let img = image::load_from_memory_with_format(tex, image::ImageFormat::Tga).unwrap();

    let tex = DynTex {
        width: img.width() as u16,
        height: img.height() as u16,
        data: img.to_rgba8().to_vec(),
    };
    rend.upload_texture(None, &tex)
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    let mut rend = Render::new(&window);

    //upload and pack model
    let model = Model {
        mesh: rend.upload_mesh(
            Some("Triangle Mesh"),
            &MeshPtr::new(include_bytes_aligned!(4, "sphere.mesh")),
        ),
        diffuse: load_tga(&mut rend, include_bytes!("diffuse.tga")),
        rough: load_tga(&mut rend, include_bytes!("specular.tga")),
        metal: load_tga(&mut rend, include_bytes!("specular.tga")),
        normal: load_tga(&mut rend, include_bytes!("normal.tga")),
    };

    // generate a camera
    let cam = Mat4::look_at_rh(
        Vec3::new(-2.0, -2.0, 2.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::Z,
    );

    let light_pos = [-1.0, 0.0, 1.0];

    let mut rx = 0.0;
    let mut ry = 0.0;
    let mut rz = 0.0;

    boil_winit(window, event_loop, move || {
        rx += 0.01;
        ry += 0.014;
        rz += 0.005;

        let rot = Quat::from_euler(glam::EulerRot::ZYX, rx, ry, rz);

        let mut frame = rend.frame();
        frame.set_camera(cam);
        frame.draw_mesh(&model, [-0.5, 0.0, 0.0], rot, [2.0; 3]);
        frame.draw_light(Light::Directional {
            color: [1.0, 1.0, 1.0],
            direction: light_pos,
        });
    });
}

fn boil_winit(window: Window, event_loop: EventLoop<()>, mut update: impl FnMut() + 'static) -> ! {
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                window_id,
            } if window_id == window.id() => context::resize(size.width, size.height),
            Event::RedrawRequested(..) => {
                update();
                std::thread::sleep(Duration::from_millis(16));
                window.request_redraw();
            }
            _ => (),
        }
    });
}
