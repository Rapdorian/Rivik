use std::{
    f32::consts::PI,
    mem,
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use assets::{formats::mesh::ObjMesh, load};
use egui::{
    plot::{Bar, BarChart, Plot},
    Color32,
};
use image::ImageFormat;
use log::{error, info};
use pollster::block_on;
use rivik_render::{
    context::{device, resize, surface_config},
    draw::mesh::MeshRenderable,
    filters::display::DisplayFilter,
    lights::{ambient::AmbientLight, sun::SunLight},
    load::{GpuMesh, GpuTexture},
    tracing::UiSubscriber,
    Frame, Transform,
};
use snafu::{ErrorCompat, ResultExt, Whatever};
use tracing::{debug_span, dispatcher::set_global_default, Dispatch};
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
    let mesh = load("file:assets/fighter_smooth.obj", GpuMesh(ObjMesh))
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
    let mesh_bundle = MeshRenderable::new(mesh, &transform, tex);

    let mut i = 0;

    let ambient = AmbientLight::new(0.01, 0.01, 0.01);
    let sun = SunLight::new(Vec3::new(1.0, 1.0, 1.0), Vec3::new(-1., 1., 0.));

    let display = DisplayFilter::default();

    // setup performance tracing
    let subscriber = UiSubscriber::new();
    let spans = subscriber.data();
    let dispatch = Dispatch::new(subscriber);
    set_global_default(dispatch).unwrap();
    let mut captured_trace = Arc::new(RwLock::new(None));

    // issue frame requests from another thread
    {
        let window = window.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(1000 / 60));
            window.request_redraw();
        });
    }

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
                let span_chart = spans.write().unwrap().generate_chart();

                let span = debug_span!("Draw frame");
                let _redraw = span.enter();

                i = (i + 1) % 4096;

                let span = debug_span!("Rotate Ship");
                let span = span.enter();
                model = Mat4::from_rotation_y(i as f32 / (4096.0 / (2. * PI)))
                    * Mat4::from_translation(Vec3::new(0.0, -0.7, 0.0));
                transform.update(proj, view, model);
                mem::drop(span);

                {
                    let span = debug_span!("Build frame");
                    let _e = span.enter();
                    frame.draw_geom(&mesh_bundle);
                    frame.draw_light(&sun);
                    frame.draw_light(&ambient);
                    frame.draw_filter(&display);
                }

                let span = debug_span!("Run UI");
                let ui_span = span.enter();
                // run UI
                let input = egui_winit.take_egui_input(&window);

                let output = ctx.run(input, |ctx| {
                    egui::Window::new("Frame timing").show(&ctx, |ui| {
                        if captured_trace.read().unwrap().is_none() {
                            if ui.button("Capture").clicked() {
                                *captured_trace.write().unwrap() = Some(span_chart.clone());
                            }
                        } else {
                            if ui.button("Resume").clicked() {
                                *captured_trace.write().unwrap() = None;
                            }
                        }

                        let chart1 = BarChart::new(
                            if let Some(ref chart) = *captured_trace.read().unwrap() {
                                chart.clone()
                            } else {
                                span_chart
                            },
                        )
                        .highlight(true)
                        .horizontal()
                        .width(1.0)
                        .name("Set 1");

                        Plot::new("Name of plot")
                            .data_aspect(0.5)
                            .allow_scroll(false)
                            .show_axes([false, false])
                            .show_background(false)
                            .show(ui, |plot| {
                                plot.bar_chart(chart1);
                            });
                    });
                });

                egui_winit.handle_platform_output(&window, &ctx, output.platform_output);
                let clipped_primitives = ctx.tessellate(output.shapes);
                frame.ui(&clipped_primitives, output.textures_delta);
                mem::drop(ui_span);

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
