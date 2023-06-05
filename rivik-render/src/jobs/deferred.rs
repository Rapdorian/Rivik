/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! A deferred rendering model

use wgpu::RenderBundle;

use crate::{
    context::gbuffer, draw::Bundle, pipeline::display, render_job::RenderJob, transform::Spatial,
    Frame,
};

mod geom;
mod light;

pub use geom::*;
pub use light::*;

/// Deferred Render pipeline
#[derive(Default)]
pub struct Deferred {
    light: LightPass,
}

impl Deferred {
    #[allow(missing_docs)]
    pub fn push_geom<T: Bundle + Spatial>(&mut self, r: &T) {
        self.light.push_geom(r);
    }

    #[allow(missing_docs)]
    pub fn push_light(&mut self, r: RenderBundle) {
        self.light.push_light(r);
    }
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
