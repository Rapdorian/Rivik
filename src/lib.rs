use std::{any::Any, cell::RefCell, rc::Rc, sync::mpsc::Sender, time::Duration};

use once_cell::sync::OnceCell;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub mod plugins {
    pub mod life_cycle;
    pub mod render;
}

pub struct Rivik {
    /// plugins
    plugins: Vec<Rc<RefCell<dyn Any>>>,
}

impl Rivik {
    /// Create an instance of the Rivik engine
    pub fn new() -> Self {
        // this should leave a single init plugin that handles lifecycle events
        Self { plugins: vec![] }
    }

    /// Register the plugin with rivik
    pub fn register<T: 'static>(&mut self, plugin: T) -> Rc<RefCell<T>> {
        // wrap plugin
        let plugin = Rc::new(RefCell::new(plugin));
        self.plugins.push(plugin.clone());
        plugin
    }
}

// Entry point for rivik engine
//
// Engine should be largely configured when this is called
//
// For initialization we need access to the event loop before the run
//pub fn run(life_cycle: plugins::life_cycle::LifeCycle) {
//    let event_loop = EventLoop::new();
//
//    let window = WindowBuilder::new()
//        .with_title("Rivik Engine")
//        .build(&event_loop)
//        .expect("Failed to build window");
//    let window = WINDOW.try_insert(window).unwrap();
//}
