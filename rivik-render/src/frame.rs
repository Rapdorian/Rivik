/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::borrow::Borrow;

use egui::{ClippedPrimitive, TexturesDelta};
use snafu::{Backtrace, ResultExt, Snafu};
use tracing::{debug, debug_span, instrument};
use wgpu::{Color, CommandEncoder, RenderBundle, SurfaceError, SurfaceTexture, TextureView};

use crate::{
    context::{device, egui_render, gbuffer, queue, surface, surface_config},
    filters::DisplayFilter,
};

/// An error constructing a frame
#[derive(Debug, Snafu)]
#[snafu(display("Failed to create the next frame"))]
pub struct FrameError {
    source: SurfaceError,
    backtrace: Backtrace,
}

/// Handle's requesting and drawing to a frame
#[derive(Debug)]
pub struct Frame<'a> {
    pub(crate) frame: SurfaceTexture,
    pub(crate) frame_view: TextureView,
    pub(crate) encoder: CommandEncoder,
    geom: Vec<&'a RenderBundle>,
    lights: Vec<&'a RenderBundle>,
    filters: Vec<&'a RenderBundle>,
    ui: Option<(&'a [ClippedPrimitive], TexturesDelta)>,
}

/// An object that can be drawn to a frame
pub trait Drawable {
    /// Fetch a render bundle that draws this object
    fn bundle(&self) -> &RenderBundle;
}

impl<T> Drawable for T
where
    T: Borrow<RenderBundle>,
{
    fn bundle(&self) -> &RenderBundle {
        self.borrow()
    }
}

impl<'a> Frame<'a> {
    /// Finalize this frame and draw it to screen
    #[instrument(skip(self))]
    pub fn present(mut self) {
        // TODO: Handle camera transform setup
        {
            let geom_span = debug_span!("Geometry render pass");
            let _e = geom_span.enter();
            let mut rpass = gbuffer().rpass(&mut self.encoder, Some(Color::BLACK));
            rpass.execute_bundles(self.geom);
        }

        {
            let span = debug_span!("Lighting render pass");
            let _e = span.enter();
            let mut rpass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Lighting"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &gbuffer().hdr_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            rpass.execute_bundles(self.lights);
        }

        {
            let span = debug_span!("Filter render pass");
            let _e = span.enter();
            // really ugly way of generating a default list of filters but only creating them if there
            // is an empty filter list.
            //
            // This is ugly because we are regenerating the list every frame.
            // TODO: this should probably be recorded once and statically cached
            let display = if self.filters.is_empty() {
                Some(DisplayFilter::default())
            } else {
                None
            };

            let filters = if !self.filters.is_empty() {
                self.filters
            } else {
                vec![display.as_ref().unwrap().bundle()]
            };

            {
                let mut rpass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Filters"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.frame_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                rpass.execute_bundles(filters);
            }
        }

        // render UI pass
        if let Some((clipped_primitives, textures_delta)) = self.ui {
            let span = debug_span!("Render UI");
            let _e = span.enter();
            let mut renderer = egui_render().write().unwrap();
            // update textures
            for (id, image) in textures_delta.set {
                renderer.update_texture(device(), queue(), id, &image);
            }

            let screen_descriptor = {
                let config = surface_config().read().unwrap();
                egui_wgpu::renderer::ScreenDescriptor {
                    size_in_pixels: [config.width, config.height],
                    pixels_per_point: 1.0,
                }
            };

            let _ = renderer.update_buffers(
                device(),
                queue(),
                &mut self.encoder,
                &clipped_primitives,
                &screen_descriptor,
            );

            {
                let mut rpass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.frame_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                    label: Some("egui_render"),
                });

                renderer.render(&mut rpass, &clipped_primitives, &screen_descriptor);
            }

            for id in textures_delta.free {
                renderer.free_texture(&id);
            }
        }

        let span = debug_span!("GPU time");
        let _e = span.enter();

        let _ = queue().submit(Some(self.encoder.finish()));
        debug!("Presenting Frame");
        self.frame.present();
    }

    /// Try to fetch a new frame
    /// The frame will be renderered and presented when this object is dropped
    pub fn new() -> Result<Frame<'a>, FrameError> {
        // get next frame
        let frame = surface().get_current_texture().context(FrameSnafu)?;
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let encoder =
            device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        Ok(Frame {
            frame,
            frame_view,
            encoder,
            geom: vec![],
            lights: vec![],
            filters: vec![],
            ui: None,
        })
    }

    /// Draw a geometry object to the internal g-buffer
    pub fn draw_geom(&mut self, geom: &'a dyn Drawable) {
        self.geom.push(geom.bundle());
    }

    /// Add a light to this frame
    pub fn draw_light(&mut self, light: &'a dyn Drawable) {
        self.lights.push(light.bundle());
    }

    /// Add a post-processing filter to this frame
    pub fn draw_filter(&mut self, filter: &'a dyn Drawable) {
        self.filters.push(filter.bundle());
    }

    /// Draw an EGUI ui to this frame
    pub fn ui(&mut self, clip_prim: &'a [ClippedPrimitive], textures: TexturesDelta) {
        // store these values in self so we can render the UI later
        self.ui = Some((clip_prim, textures));
    }
}
