//! Root game lifecycle plugin

use std::{mem, process::exit, time::Duration};

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowId},
};

/// Main entry point
///
/// This currently assume you'll only have a single window
pub struct LifeCycle {
    event_loop: EventLoop<()>,
    window: Window,
    // event queues
    init: Vec<Box<dyn Fn()>>,
    update: Vec<Box<dyn Fn()>>,
    draw: Vec<Box<dyn Fn()>>,
    shutdown: Vec<Box<dyn Fn()>>,
}

impl LifeCycle {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        Self {
            window: Window::new(&event_loop).unwrap(),
            event_loop: event_loop,
            init: vec![],
            update: vec![],
            draw: vec![],
            shutdown: vec![],
        }
    }

    pub fn init(&self) {
        for callback in &self.init {
            callback();
        }
    }

    pub fn update(&self) {
        for callback in &self.update {
            callback();
        }
    }

    pub fn draw(&self) {
        for callback in &self.draw {
            callback();
        }
    }

    pub fn shutdown(&self) {
        for callback in &self.shutdown {
            callback();
        }
        exit(-1);
    }

    pub fn on_init(&mut self, f: impl Fn() + 'static) {
        self.init.push(Box::new(f));
    }

    pub fn on_update(&mut self, f: impl Fn() + 'static) {
        self.update.push(Box::new(f));
    }

    pub fn on_draw(&mut self, f: impl Fn() + 'static) {
        self.draw.push(Box::new(f));
    }

    pub fn on_shutdown(&mut self, f: impl Fn() + 'static) {
        self.shutdown.push(Box::new(f));
    }
    pub fn event_loop(&self) -> &EventLoop<()> {
        &self.event_loop
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn run(mut self) -> ! {
        self.init();

        // borrow check reasons
        let event_loop = mem::take(&mut self.event_loop);

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent { window_id, event } if window_id == self.window.id() => {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Wait;
                        }
                        _ => {}
                    }
                }
                Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                    self.draw();
                    // TODO: Better update loop/timestep
                    std::thread::sleep(Duration::from_millis(16));
                    self.window.request_redraw();
                }
                Event::LoopDestroyed => self.shutdown(),
                _ => {}
            }
        });
    }
}
