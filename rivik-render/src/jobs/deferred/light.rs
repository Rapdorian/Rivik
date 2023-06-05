use wgpu::RenderBundle;

use crate::{context::gbuffer, draw::Bundle, render_job::RenderJob, transform::Spatial, Frame};

use super::GeomPass;

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
    pub fn push_geom<T: Bundle + Spatial>(&mut self, r: &T) {
        self.geom.push(r);
    }

    /// Add a light to the render
    pub fn push_light(&mut self, r: RenderBundle) {
        self.nodes.push(r);
    }
}
