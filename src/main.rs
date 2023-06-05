//! Basic version of the engine for testing

use std::{fs, time::Duration};

use glam::{EulerRot, Mat4, Quat, Vec3};
use include_bytes_aligned::include_bytes_aligned;
use legion::{
    component,
    serialize::Canon,
    system,
    systems::CommandBuffer,
    world::{Entry, SubWorld},
    Entity, EntityStore, Query, Registry, Resources, Schedule, World,
};
use rivik_deferred::{
    types::{light::Light, mesh::MeshPtr, tex::DynTex, Frame, Model, Renderer},
    Render,
};
use serde::{Deserialize, Serialize};
use winit::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// plan:
// For now we'll just implement a simple game with legion and see where it takes us design wise

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
}

#[system(for_each)]
fn load_model(
    entity: &Entity,
    cmd: &mut CommandBuffer,
    model: &mut Model<String>,
    loaded: Option<&Model<u32>>,
    #[resource] render: &mut Render,
) {
    if loaded.is_some() {
        // don't load this model
        return;
    }
    // create a new model
    let loaded = Model {
        mesh: render.upload_mesh(None, &MeshPtr::new(&fs::read(&model.mesh).unwrap())),
        diffuse: load_tga(render, &fs::read(&model.diffuse).unwrap()),
        metal: load_tga(render, &fs::read(&model.metal).unwrap()),
        rough: load_tga(render, &fs::read(&model.rough).unwrap()),
        normal: load_tga(render, &fs::read(&model.normal).unwrap()),
    };

    cmd.add_component(*entity, loaded);
    //cmd.remove_component::<Model<String>>(*entity);
}

#[system]
fn render_sys(
    #[resource] render: &mut Render,
    world: &mut SubWorld,
    query: &mut Query<(&Transform, &Model<u32>)>,
) {
    // create a frame and queue everything
    // This may not be the best way to do this.
    // Creating a frame resource that is updated per frame may be better
    let mut frame = render.frame();
    frame.draw_light(Light::Directional {
        color: [1.0; 3],
        direction: [-1., 0., 1.],
    });

    let cam = Mat4::look_at_rh(Vec3::new(-2.0, -2.0, 2.0), Vec3::ZERO, Vec3::Z);
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

#[system(for_each)]
fn rotate(transform: &mut Transform) {
    transform.rotation *= Quat::from_euler(EulerRot::XYZ, 0.02, 0.05, 0.03);
}

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
    // setup winit
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Rivik Engine")
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    // init game
    let mut world = World::default();
    let mut res = Resources::default();

    // setup game resources
    res.insert(Render::new(&window));
    world.extend([(
        Transform {
            position: Vec3::ZERO,
            rotation: Quat::default(),
            scale: Vec3::ONE,
        },
        Model {
            mesh: String::from("rivik-deferred/examples/sphere.mesh"),
            diffuse: String::from("rivik-deferred/examples/diffuse.tga"),
            rough: String::from("rivik-deferred/examples/specular.tga"),
            metal: String::from("rivik-deferred/examples/specular.tga"),
            normal: String::from("rivik-deferred/examples/normal.tga"),
        },
    )]);

    let mut registry = Registry::<String>::default();
    registry.register::<Transform>("transform".to_string());
    registry.register::<Model<String>>("model".to_string());
    registry.on_unknown(legion::serialize::UnknownType::Ignore);

    let entity_serializer = Canon::default();

    // start event pump
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    device_id,
                    input,
                    is_synthetic,
                } => {
                    if Some(VirtualKeyCode::Tab) == input.virtual_keycode
                        && input.state == ElementState::Pressed
                    {
                        // print serialized world
                        let ser = toml::to_string_pretty(&world.as_serializable(
                            component::<Transform>(),
                            &registry,
                            &entity_serializer,
                        ))
                        .unwrap();
                        println!("{ser}");
                    }
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                let mut update = Schedule::builder().add_system(rotate_system()).build();
                update.execute(&mut world, &mut res);
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                // create draw schedule
                let mut draw_sched = Schedule::builder()
                    .add_system(load_model_system())
                    .add_system(render_sys_system())
                    .build();
                draw_sched.execute(&mut world, &mut res);

                // shitty framerate control
                std::thread::sleep(Duration::from_millis(16));
                window.request_redraw();
            }
            _ => {}
        }
    });
}
