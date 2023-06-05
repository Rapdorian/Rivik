/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::borrow::Borrow;

use egui::{ClippedPrimitive, TexturesDelta};
use snafu::{Backtrace, ResultExt, Snafu};
use tracing::{debug, instrument};
use wgpu::{
    CommandEncoder, RenderBundle, RenderPass, RenderPassDescriptor, SurfaceError, SurfaceTexture,
    TextureView,
};

use crate::context::{device, queue, surface};

/// An error constructing a frame
#[derive(Debug, Snafu)]
#[snafu(display("Failed to create the next frame"))]
pub struct FrameError {
    source: SurfaceError,
    backtrace: Backtrace,
}

/// Handle's requesting and drawing to a frame
///
/// Provides access to queueing render passes
#[derive(Debug)]
pub struct Frame<'a> {
    pub(crate) frame: SurfaceTexture,
    pub(crate) frame_view: TextureView,
    pub(crate) encoder: CommandEncoder,
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
    pub fn present(self) {
        let _ = queue().submit(Some(self.encoder.finish()));
        debug!("Presenting Frame");
        self.frame.present();
    }

    /// Create a render pass inside this frame
    pub fn render_pass(&'a mut self, desc: RenderPassDescriptor<'a, 'a>) -> RenderPass<'a> {
        self.encoder.begin_render_pass(&desc)
    }

    /// Start a custom renderpass
    pub fn render<T: super::render_job::RenderPass>(&'a mut self) -> T {
        T::begin(self)
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
            ui: None,
        })
    }

    /// Draw an EGUI ui to this frame
    pub fn ui(&mut self, clip_prim: &'a [ClippedPrimitive], textures: TexturesDelta) {
        // store these values in self so we can render the UI later
        self.ui = Some((clip_prim, textures));
    }
}
