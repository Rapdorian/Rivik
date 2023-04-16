/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! A library the provides a simple app structure for Rivik

//
// I need to do a bit of thinking on API for adding a manipulating objects in the scenegraph.
// I think I'll just handle a mapping from transform->renderbundle as the scenegraph
// so have a function with a similar API to Frame::draw_* but also returns a handle to a transform
// any game logic can be handled by the user using a scenegraph handle to refer to the renderable

use std::{
    any::Any,
    f32::consts::FRAC_PI_2,
    marker::PhantomData,
    sync::{Arc, RwLock},
    thread,
    time::{Duration, Instant},
};

pub use egui_winit::egui;
use egui_winit::egui::{plot::Plot, TopBottomPanel};
use glam::{Mat4, Vec3};
use legion::{systems::Builder, Resources, Schedule, World};
use pollster::block_on;
use render::{
    context::{resize, surface_config},
    tracing::{display_traces, generate_chart},
    transform::Spatial,
    Drawable, Frame,
};
pub use rivik_assets as assets;
pub use rivik_render as render;
pub use rivik_scene as scene;
pub use winit;

use scene::Node;
use tracing::debug_span;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod components {
    use crate::Renderable;

    pub struct Light(pub Box<dyn Renderable + Send + Sync>);
    pub struct Geom(pub Box<dyn Renderable + Send + Sync>);
}

pub(crate) mod systems {
    pub(crate) mod render {
        use std::sync::{Arc, RwLock};

        use egui_winit::egui::{
            self,
            plot::{Bar, Plot},
            FullOutput, TopBottomPanel,
        };
        use glam::Mat4;
        use legion::{system, Resources, World};
        use rivik_render::{
            tracing::{display_traces, generate_chart},
            Frame,
        };
        use rivik_scene::Node;
        use tracing::debug_span;
        use winit::window::Window;

        use crate::{
            components::{Geom, Light},
            App,
        };

        #[system]
        pub(crate) fn render(
            #[resource] state: &mut egui_winit::State,
            #[resource] ctx: &egui::Context,
            #[resource] window: &Arc<Window>,
            #[resource] output: &Option<FullOutput>,
        ) {
            let mut frame = Frame::new().unwrap();

            // Render EGUI
            let clipped_primitives;
            if let Some(output) = output {
                let ui_span = debug_span!("Drawing UI");
                let ui_span = ui_span.enter();
                state.handle_platform_output(&window, &ctx, output.platform_output.clone());
                clipped_primitives = ctx.tessellate(output.shapes.clone());
                frame.ui(&clipped_primitives, output.textures_delta.clone());
                drop(ui_span);
            }
            frame.present();
        }

        #[tracing::instrument(skip_all)]
        #[system]
        pub(crate) fn setup_ui<A: App + 'static>(
            #[resource] state: &mut egui_winit::State,
            #[resource] ctx: &egui::Context,
            #[resource] window: &Arc<Window>,
            #[resource] app: &mut A,
            #[resource] trace: &mut Option<(Vec<Bar>, Vec<Bar>)>,
            #[resource] output: &mut Option<FullOutput>,
        ) {
            let trace_chart = generate_chart();
            // draw ui
            let input = state.take_egui_input(&window);
            *output = Some(ctx.run(input, |ctx| {
                let span = debug_span!("Drawing traces");
                let _span = span.enter();
                TopBottomPanel::bottom("Frame timing").show(ctx, |ui| {
                    if trace.is_none() {
                        if ui.button("Capture").clicked() {
                            *trace = Some(trace_chart.clone());
                        }
                    } else if ui.button("Resume").clicked() {
                        *trace = None;
                    }
                    let trace = trace.as_ref().unwrap_or(&trace_chart);

                    Plot::new("Frame Timing")
                        .data_aspect(0.1)
                        .view_aspect(10.0)
                        .show_axes([false, false])
                        .show_background(false)
                        .show(ui, |plot| {
                            display_traces(plot, trace.0.clone(), trace.1.clone());
                        });
                });
                let span = debug_span!("Drawing user UI");
                let _span = span.enter();
                app.ui(ctx)
            }));
        }
    }
}

trait Renderable: Drawable + Spatial + Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
impl<T> Renderable for T
where
    T: Drawable + Spatial + Any,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// enum RenderPassType {
//     Geom,
//     Light,
//     #[allow(dead_code)]
//     Filter,
// }
//
// pub struct Handle<T> {
//     inner: usize,
//     pass: RenderPassType,
//     ty: PhantomData<T>,
// }
//
// impl<T: Drawable + 'static> Handle<T> {
//     pub fn get<'a>(&self, ctx: &'a Context) -> &'a T {
//         match self.pass {
//             RenderPassType::Geom => ctx.geom[self.inner].0.as_any().downcast_ref().unwrap(),
//             _ => todo!(),
//         }
//     }
//
//     pub fn get_mut<'a>(&self, ctx: &'a mut Context) -> &'a mut T {
//         match self.pass {
//             RenderPassType::Geom => ctx.geom[self.inner].0.as_any_mut().downcast_mut().unwrap(),
//             _ => todo!(),
//         }
//     }
//
//     pub fn transform<'a>(&self, ctx: &'a Context) -> &'a Arc<RwLock<Node<Mat4>>> {
//         match self.pass {
//             RenderPassType::Geom => &ctx.geom[self.inner].1,
//             _ => todo!(),
//         }
//     }
// }

#[derive(Default)]
pub struct Context {
    world: World,
    resources: Resources,
    pub camera: Mat4,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub show_trace: bool,

    pub update_step: f32,
    pub framerate: u8,
}

impl Context {
    pub fn world(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn resources(&mut self) -> &mut Resources {
        &mut self.resources
    }
}

pub trait App {
    fn name() -> &'static str {
        "Rivik App"
    }
    fn update(&mut self, _ctx: &mut Context, _schedule: &mut Builder) {}

    // Event will be handled differntly
    fn on_event(&mut self, _event: &WindowEvent) {}
    fn ui(&mut self, _ctx: &egui::Context) {}
    fn init(_ctx: &mut Context) -> Self;
}

pub fn run<A: App + Send + Sync + 'static>() -> ! {
    let event_loop = EventLoop::new();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title(A::name())
            .build(&event_loop)
            .expect("Failed to build window"),
    );

    block_on(render::init(&window, (1920, 1080))).expect("Failed to init Rivik");

    let mut egui_state = egui_winit::State::new(&event_loop);
    let egui_ctx = egui::Context::default();

    let mut scene = Context::default();
    let mut render_resources = Resources::default();

    scene.camera = Mat4::look_at_rh(Vec3::new(2.0, 2.0, 0.0), Vec3::default(), Vec3::Y);
    scene.fov = FRAC_PI_2;
    scene.near = 0.1;
    scene.far = 1_000.0;
    scene.framerate = 60;
    scene.update_step = 1.0 / 60.0;
    let mut app = A::init(&mut scene);

    let mut captured_trace = None;
    let mut last_frame_time = Instant::now();

    let mut proj = Mat4::perspective_rh(scene.fov, 16.0 / 9.0, scene.near, scene.far);

    render_resources.insert(egui_winit::State::new(&event_loop));
    render_resources.insert(egui::Context::default());
    render_resources.insert(A::init(&mut scene));
    render_resources.insert(Arc::clone(&window));

    // build render schedule
    let mut render_schedule = Schedule::builder()
        .add_system(todo!("Update Transforms"))
        .add_system(systems::render::setup_ui_system::<A>())
        .flush()
        .add_system(systems::render::render_system())
        .build();

    // issue frame requests from another thread
    {
        let window = window.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(1000 / scene.framerate as u64));
            window.request_redraw();
        });
    }

    let mut dt = 0.0;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => {
                let resp = egui_state.on_event(&egui_ctx, &event);

                if !resp.consumed {
                    match event {
                        WindowEvent::Resized(size) => {
                            resize(size.width, size.height);
                            // re-compute projection matrix
                            let aspect = {
                                let config = surface_config().read().unwrap();
                                config.width as f32 / config.height as f32
                            };

                            proj = Mat4::perspective_rh(scene.fov, aspect, scene.near, scene.far);
                        }
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        event => app.on_event(&event),
                    }
                }
            }
            Event::RedrawRequested(..) => {
                let mut frame = Frame::new().unwrap();

                // generate dt
                dt += last_frame_time.elapsed().as_secs_f32();
                last_frame_time = Instant::now();

                let trace_chart = generate_chart();

                {
                    println!("TODO: Draw scene");
                    // let span = debug_span!("Preparing Scenegraph");
                    // let _span = span.enter();
                    // for (drawable, transform) in &scene.geom {
                    //     // update transform buffer
                    //     drawable.transform().update(
                    //         proj,
                    //         scene.camera,
                    //         transform.read().unwrap().global(),
                    //     );
                    //     frame.draw_geom(drawable.bundle());
                    // }

                    // for (light, transform) in &scene.lights {
                    //     // update transform buffer
                    //     light.transform().update(
                    //         proj,
                    //         scene.camera,
                    //         transform.read().unwrap().global(),
                    //     );
                    //     frame.draw_light(light.bundle());
                    // }
                }

                let ui_span = debug_span!("Drawing EGUI");
                let ui_span = ui_span.enter();

                let input;
                {
                    let span = debug_span!("Finding UI inputs");
                    let _span = span.enter();
                    input = egui_state.take_egui_input(&window);
                }
                let output = egui_ctx.run(input, |ctx| {
                    if scene.show_trace {
                        let span = debug_span!("Drawing traces");
                        let _span = span.enter();
                        TopBottomPanel::bottom("Frame timing").show(ctx, |ui| {
                            if captured_trace.is_none() {
                                if ui.button("Capture").clicked() {
                                    captured_trace = Some(trace_chart.clone());
                                }
                            } else if ui.button("Resume").clicked() {
                                captured_trace = None;
                            }

                            let spans = captured_trace.clone().unwrap_or(trace_chart);
                            Plot::new("Frame Timing")
                                .data_aspect(0.1)
                                .view_aspect(10.0)
                                .show_axes([false, false])
                                .show_background(false)
                                .show(ui, |plot| {
                                    display_traces(plot, spans.0, spans.1);
                                });
                        });
                    }
                    let span = debug_span!("Drawing user UI");
                    let _span = span.enter();
                    app.ui(ctx)
                });
                egui_state.handle_platform_output(&window, &egui_ctx, output.platform_output);
                let clipped_primitives = egui_ctx.tessellate(output.shapes);
                frame.ui(&clipped_primitives, output.textures_delta);
                drop(ui_span);

                frame.present();

                while dt >= scene.update_step {
                    let span = debug_span!("Updating scene", dt);
                    let _e = span.enter();
                    // update game logic

                    let mut schedule = Schedule::builder();
                    app.update(&mut scene, &mut schedule);
                    let mut schedule = schedule.build();
                    schedule.execute(&mut scene.world, &mut scene.resources);

                    dt -= scene.update_step;
                }
            }
            _ => {}
        }
    })
}
