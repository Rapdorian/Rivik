use std::{
    f32::consts::FRAC_2_PI,
    mem,
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use assets::{formats::mesh::ObjMesh, load};
use egui::plot::Plot;
use image::ImageFormat;
use pollster::block_on;
use rivik_render::{
    context::{resize, surface_config},
    draw::{self, pixel_mesh::vertex_buffer, Bundle, PixelMesh as Mesh},
    filters::DisplayFilter,
    jobs::deferred::{Deferred, LightPass},
    lights::{AmbientLight, SunLight},
    load::{GpuMesh, GpuTexture},
    render_job::RenderJob,
    tracing::{display_traces, generate_chart, UiSubscriber},
    transform::Spatial,
    Frame, Transform,
};
use snafu::{ErrorCompat, ResultExt, Whatever};
use tracing::{debug_span, dispatcher::set_global_default, Dispatch};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, Registry};
use ultraviolet::{Mat4, Vec3, Vec4};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

async fn run() -> Result<(), Whatever> {
    // init window
    let event_loop = EventLoop::new();
    let proxy = event_loop.create_proxy();
    env_logger::init();
    let window = Arc::new(Window::new(&event_loop).whatever_context("Failed to build window")?);
    rivik_render::init(&window, (1920, 1080)).await;

    let mut egui_winit = egui_winit::State::new(&event_loop);
    let mut ctx = egui::Context::default();

    // load a mesh
    let mesh = load(
        "file:assets/fighter_smooth.obj",
        GpuMesh(ObjMesh, vertex_buffer),
    )
    .whatever_context("Failed to fetch fighter ship")?;
    let tex = load(
        "file:assets/fighter.albedo.png",
        GpuTexture(ImageFormat::Png),
    )
    .whatever_context("Failed to fetch fighter ship texture")?;

    let aspect = {
        let config = surface_config().read().unwrap();
        config.width as f32 / config.height as f32
    };

    let eye = Vec3::new(2.0, 2.0, 2.0);
    let focus = Vec3::new(0.0, 0.0, 0.0);

    let mut proj = ultraviolet::projection::perspective_vk(80.0, aspect, 0.1, 100.0);
    let mut view = ultraviolet::Mat4::look_at(eye, focus, Vec3::unit_y());

    let mut model = ultraviolet::Mat4::identity();

    let transform = Transform::new(proj, view, model);
    let mesh_bundle = Mesh::new(mesh, tex);

    let mut i = 0;

    let ambient = AmbientLight::new(0.01, 0.01, 0.01);
    let sun_dir = Vec3::new(1.0, 0.6, 1.0);
    let sun = SunLight::new(Vec3::new(1.0, 1.0, 1.0), Vec3::new(-1., 1., 0.));

    let display = DisplayFilter::default();

    // setup performance tracing

    set_global_default(Dispatch::new(
        Registry::default().with(UiSubscriber::default()),
    ));

    let mut captured_trace = Arc::new(RwLock::new(None));

    // issue frame requests from another thread
    {
        let window = window.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(1000 / 60));
            window.request_redraw();
        });
    }

    let mut pipeline = Deferred::default();
    pipeline.push_geom(mesh_bundle.bundle());
    pipeline.push_light(ambient.bundle());
    pipeline.push_light(sun.bundle());

    event_loop.run(move |event, _, control_flow| {
        let span = debug_span!("Event Callback", ?event);
        let _e = span.enter();
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, .. } => {
                let resp = egui_winit.on_event(&ctx, &event);

                if !resp.consumed {
                    match event {
                        WindowEvent::Resized(size) => {
                            resize(size.width, size.height);

                            // re-compute projection matrix
                            let aspect = {
                                let config = surface_config().read().unwrap();
                                config.width as f32 / config.height as f32
                            };

                            proj =
                                ultraviolet::projection::perspective_vk(80.0, aspect, 0.1, 100.0);
                        }
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(..) => {
                let mut frame = Frame::new().unwrap();
                // fetch span chart from last frame
                let span_chart = generate_chart();

                let span = debug_span!("Draw frame");
                let _redraw = span.enter();

                i = (i + 1) % 4096;

                let span = debug_span!("Rotate Ship");
                let span = span.enter();
                // model = Mat4::from_rotation_y(i as f32 / (4096.0 / (2. * PI)))
                //     * Mat4::from_translation(Vec3::new(0.0, -0.7, 0.0));

                view = Mat4::look_at(
                    (Mat4::from_rotation_y(((i as f32) / 4096.0) * FRAC_2_PI * 10.0)
                        * Vec4::new(3.0, 2.5, 0.0, 1.0))
                    .xyz(),
                    focus,
                    Vec3::unit_y(),
                );

                mesh_bundle.transform().update(proj, view, model);
                sun.transform().update(proj, view, model);
                mem::drop(span);
                pipeline.run(&mut frame);

                let span = debug_span!("Handle egui");
                let ui_span = span.enter();
                // run UI
                let span = debug_span!("Fetching UI inputs");
                let input_span = span.enter();
                let input = egui_winit.take_egui_input(&window);
                drop(input_span);

                let output = ctx.run(input, |ctx| {
                    let span = debug_span!("Running UI");
                    let span = span.enter();
                    egui::TopBottomPanel::bottom("Frame timing").show(&ctx, |ui| {
                        if captured_trace.read().unwrap().is_none() {
                            if ui.button("Capture").clicked() {
                                *captured_trace.write().unwrap() = Some(span_chart.clone());
                            }
                        } else {
                            if ui.button("Resume").clicked() {
                                *captured_trace.write().unwrap() = None;
                            }
                        }

                        let spans = if let Some(ref chart) = *captured_trace.read().unwrap() {
                            chart.clone()
                        } else {
                            span_chart
                        };

                        Plot::new("Name of plot")
                            .data_aspect(0.1)
                            .allow_scroll(false)
                            //.legend(Legend::default())
                            .show_axes([false, false])
                            .show_background(false)
                            .show(ui, |plot| {
                                display_traces(plot, spans.0, spans.1);
                            });
                    });
                });

                let span = debug_span!("Drawing UI");
                let draw_span = span.enter();
                egui_winit.handle_platform_output(&window, &ctx, output.platform_output);
                let clipped_primitives = ctx.tessellate(output.shapes);
                frame.ui(&clipped_primitives, output.textures_delta);
                drop(draw_span);
                drop(ui_span);

                frame.present();
            }

            _ => {}
        }
    })
}

fn main() {
    if let Err(e) = block_on(run()) {
        eprintln!("Error: {}", e);

        if e.iter_chain().skip(1).next().is_some() {
            eprintln!("\nCaused by:");
        }

        for (n, err) in e.iter_chain().skip(1).enumerate() {
            eprintln!("\t{n}: {err}");
        }

        if let Some(backtrace) = e.backtrace() {
            color_backtrace::BacktracePrinter::new()
                .print_trace(backtrace, &mut color_backtrace::default_output_stream())
                .unwrap();
        }
    }
}
