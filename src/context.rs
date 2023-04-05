//! This module contains functions for initializing and accesing the backing renderer.

use std::sync::RwLock;

use egui_wgpu::Renderer;
use once_cell::sync::OnceCell;
use reerror::{conversions::failed_precondition, Context, Result};
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::window::Window;

use crate::pipeline::GBuffer;

static WGPU_DEVICE: OnceCell<Device> = OnceCell::new();
static WGPU_SURFACE: OnceCell<Surface> = OnceCell::new();
static WGPU_QUEUE: OnceCell<Queue> = OnceCell::new();
static G_BUFFER: OnceCell<GBuffer> = OnceCell::new();
static WGPU_SURF_CONF: OnceCell<RwLock<SurfaceConfiguration>> = OnceCell::new();
static EGUI_RENDER: OnceCell<RwLock<Renderer>> = OnceCell::new();

/// Fetches the GBuffer being used by this renderer
pub fn gbuffer() -> &'static GBuffer {
    G_BUFFER.get().expect("WGPU should be initialized")
}

/// Fetches the WGPU Queue instance
pub fn queue() -> &'static Queue {
    WGPU_QUEUE.get().expect("WGPU should be initialized")
}

/// Fetches the WGPU Surface that is being rendered to
pub fn surface() -> &'static Surface {
    WGPU_SURFACE.get().expect("WGPU should be initialized")
}

/// Fetches the WGPU Device instance
pub fn device() -> &'static Device {
    WGPU_DEVICE.get().expect("WGPU should be initialized")
}

/// Fetches the config for the current WGPU surface
pub fn surface_config() -> &'static RwLock<SurfaceConfiguration> {
    WGPU_SURF_CONF.get().expect("WGPU should be initialized")
}

pub fn egui_render() -> &'static RwLock<Renderer> {
    EGUI_RENDER.get().expect("WGPU should be initialized")
}

/// Resize the WGPU surface
pub fn resize(width: u32, height: u32) {
    let config = surface_config();
    let mut config = config.write().unwrap();
    config.width = width;
    config.height = height;
    surface().configure(device(), &config);
}

/// Initialize the renderer
pub async fn init(window: &Window, gbuffer_size: (u32, u32)) -> Result<()> {
    // create stuff
    let size = window.inner_size();
    let instance = wgpu::Instance::default();
    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let surface = WGPU_SURFACE
        .try_insert(surface)
        .map_err(|_| failed_precondition("WGPU already initialized"))?;
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(surface),
        })
        .await
        .context("Failed to find an appropriate adapter")?;

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::PUSH_CONSTANTS,
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                limits: wgpu::Limits {
                    max_push_constant_size: 128,
                    ..wgpu::Limits::downlevel_defaults()
                },
            },
            None,
        )
        .await
        .unwrap();

    let _ = WGPU_QUEUE
        .try_insert(queue)
        .map_err(|_| failed_precondition("WGPU already intiialized"))?;

    let device = WGPU_DEVICE
        .try_insert(device)
        .map_err(|_| failed_precondition("WGPU already initialized"))?;

    // configure surface
    let config = surface
        .get_default_config(&adapter, size.width, size.height)
        .unwrap();
    let swapchain_format = config.format;

    surface.configure(device, &config);
    let _ = WGPU_SURF_CONF
        .try_insert(RwLock::new(config))
        .map_err(|_| failed_precondition("WGPU already initialized"))?;

    let _ = G_BUFFER
        .try_insert(GBuffer::new(gbuffer_size.0, gbuffer_size.1))
        .map_err(|_| failed_precondition("WGPU already initialized"))?;

    let Ok(_) = EGUI_RENDER.try_insert(RwLock::new(Renderer::new(
        device,
        swapchain_format,
        None,
        1,
    ))) else {
        unreachable!("This invariant has already been checked")
    };
    Ok(())
}
