use std::{
    ops::Deref,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use input::Input;
use legion::{systems::Builder, Resources, World};
use scene::Scene;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub mod input {
    mod handler;
    pub use handler::*;
    pub mod keys;
}
pub mod load;
pub mod render;
pub mod scene;

pub mod collision;
pub mod physics {
    mod body;
    mod kinematic;

    pub use body::*;
    pub use kinematic::*;
}

// TODO: Consider alternate ECS systems (maybe flecs)

// We'll need a physics module
// as an mvp we'll need a few components
// - RigidBody
// - StaticBody
// - Kinematics
// The rigid and static body components will track basic information about the physicality of the
// body (elasticity/mass)
// The kinematics component will track velocity and other movement based values
//
// the module should also include some basic systems for interacting with these components.
// for example, there should be a system to adjust a kinematics component when it interacts with a
// static or rigid body.
// Maybe a simple gravity system since most games will use the same model for gravity.
// There should also be an integration system that will add the values of the kinematics into the
// object's transform
//
// The physics engine should also be able to issue events that can be subscribed to, specifically
// collision events. These events should also report the force of a collision.
//
// There should also be a fairly simple API for checking for collisions without referenceing the
// physics components. For instance I'd like to reuse the collision engine for audio regions and
// maybe other user specified trigger zones. The broad phase detection needs to be usable from the
// ECS.
//
// pub mod physics

pub struct DeltaTime(f32);
impl Deref for DeltaTime {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Run a scene
pub fn start<S: Scene>(mut scene: S) {
    // startup the engine
    // TODO: For now we will provide a default set of components (Render/Windowing/Scene)
    // TODO: In the future we should make the base engine components configurable
    let event_loop = EventLoop::new();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Rivik Engine")
            .with_resizable(false)
            .build(&event_loop)
            .expect("Failed to open window"),
    );

    let mut world = World::default();
    let mut resources = Resources::default();
    scene.init(&mut world, &mut resources);

    // init resources
    let win_id = window.id();
    resources.insert(rivik_deferred::Render::new(&*window));
    resources.insert(window.clone());
    resources.insert(Input::default());

    let mut update = Builder::default();
    let mut draw = Builder::default();

    scene.update(&mut update);
    scene.draw(&mut draw);

    let mut update = update.build();
    let mut draw = draw.build();

    // Framerate
    {
        let window = window.clone();
        thread::spawn(move || loop {
            // TODO: Better approximate of 60 fps
            thread::sleep(Duration::from_millis(16));
            window.request_redraw();
        });
    }

    // window
    //     .set_cursor_grab(winit::window::CursorGrabMode::Confined)
    //     .unwrap();
    // window.set_cursor_visible(false);

    let mut last_frame_time = Instant::now();

    // startup event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { window_id, event } if window_id == win_id => {
                {
                    // feed event into the input handler
                    let mut inputs = resources.get_mut::<Input>().unwrap();
                    inputs.recv_winit(&event);
                }

                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == win_id => {
                //update.execute(&mut world, &mut resources);
                draw.execute(&mut world, &mut resources);
            }
            Event::MainEventsCleared => {
                // window
                //     .set_cursor_position(winit::dpi::LogicalPosition::new(100.0, 100.0))
                //     .unwrap();
                let frame_time = Instant::now();
                let dt = frame_time - last_frame_time;
                let dt = DeltaTime(dt.as_secs_f32());
                resources.insert(dt);
                last_frame_time = frame_time;
                update.execute(&mut world, &mut resources);
            }
            // Dispatch events
            _ => {}
        }
    });
}
