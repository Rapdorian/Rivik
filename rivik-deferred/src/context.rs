//! WGPU context
//!
//! containts handles to the WGPU API

use std::sync::RwLock;

use once_cell::sync::{Lazy, OnceCell};

static SURFACE: OnceCell<wgpu::Surface> = OnceCell::new();
static SURFACE_CONFIG: OnceCell<RwLock<wgpu::SurfaceConfiguration>> = OnceCell::new();
static DEVICE: OnceCell<wgpu::Device> = OnceCell::new();
static QUEUE: OnceCell<wgpu::Queue> = OnceCell::new();

pub fn queue() -> &'static wgpu::Queue {
    QUEUE.get().unwrap()
}

pub fn surface() -> &'static wgpu::Surface {
    SURFACE.get().unwrap()
}

pub fn device() -> &'static wgpu::Device {
    DEVICE.get().unwrap()
}

pub fn surface_config() -> wgpu::SurfaceConfiguration {
    SURFACE_CONFIG.get().unwrap().read().unwrap().clone()
}

pub fn resize(width: u32, height: u32) {
    let mut config = surface_config();
    config.width = width;
    config.height = height;

    surface().configure(device(), &config);
    *SURFACE_CONFIG.get().unwrap().write().unwrap() = config;
}

/// Initialize the core of the WGPU renderer
pub async fn init<W>(window: &W)
where
    W: raw_window_handle::HasRawDisplayHandle + raw_window_handle::HasRawWindowHandle,
{
    let instance = wgpu::Instance::default();

    let surface = unsafe { instance.create_surface(window) }.unwrap();

    let surface = SURFACE.try_insert(surface).ok().expect("Failed to init");

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(surface),
        })
        .await
        .expect("Failed to find appropriate adapter");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Deferred renderer adapter"),
                features: adapter.features(),
                limits: adapter.limits(),
            },
            None,
        )
        .await
        .unwrap();

    QUEUE.try_insert(queue).ok().unwrap();
    let device = DEVICE.try_insert(device).ok().unwrap();

    let config = surface.get_default_config(&adapter, 800, 600).unwrap();
    let swapchain_format = config.format;

    surface.configure(device, &config);

    SURFACE_CONFIG.try_insert(RwLock::new(config)).ok().unwrap();
}
