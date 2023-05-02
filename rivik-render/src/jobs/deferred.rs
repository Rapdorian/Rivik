/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! TODO: This still doesn't have a way to link render jobs together wrt storage

use wgpu::{Color, RenderBundle};

use crate::{context::gbuffer, pipeline::display, render_job::RenderJob, Frame};

/// Part of a deferred render pipeline
#[derive(Default)]
pub struct GeomPass {
    nodes: Vec<RenderBundle>,
}

impl GeomPass {
    /// Add a renderbundle to this render pass
    pub fn push(&mut self, r: RenderBundle) {
        self.nodes.push(r);
    }
}

impl RenderJob for GeomPass {
    fn begin<'a>(&self, frame: &'a mut Frame) -> wgpu::RenderPass<'a> {
        gbuffer().rpass(
            Some("Deferred Geom Pass"),
            &mut frame.encoder,
            Some(Color::BLACK),
        )
    }

    fn render<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        rpass.execute_bundles(&self.nodes);
    }
}

/// Part of a deferred render pipeline
#[derive(Default)]
pub struct LightPass {
    geom: GeomPass,
    nodes: Vec<RenderBundle>,
}

impl RenderJob for LightPass {
    fn inputs(&self) -> Vec<&dyn RenderJob> {
        vec![&self.geom]
    }

    fn begin<'a>(&self, frame: &'a mut Frame) -> wgpu::RenderPass<'a> {
        frame
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Deferred Lighting Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &gbuffer().hdr_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            })
    }

    fn render<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        rpass.execute_bundles(&self.nodes);
    }
}

impl LightPass {
    /// Add a piece of geometry to this render
    pub fn push_geom(&mut self, r: RenderBundle) {
        self.geom.push(r);
    }

    /// Add a light to the render
    pub fn push_light(&mut self, r: RenderBundle) {
        self.nodes.push(r);
    }
}

/// Deferred Render pipeline
#[derive(Default)]
pub struct Deferred {
    light: LightPass,
}

impl RenderJob for Deferred {
    fn inputs(&self) -> Vec<&dyn RenderJob> {
        vec![&self.light]
    }

    fn begin<'a>(&self, frame: &'a mut Frame) -> wgpu::RenderPass<'a> {
        frame
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Deferred HDR pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame.frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            })
    }

    fn render<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        rpass.set_pipeline(display::pipeline());
        rpass.set_bind_group(0, &gbuffer().hdr_bind, &[]);
        rpass.draw(0..7, 0..1);
    }
}
