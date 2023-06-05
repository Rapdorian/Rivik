use wgpu::{Color, RenderBundle};

use crate::{
    context::gbuffer, draw::Bundle, jobs::frustum_cull::FrustumCulled, render_job::RenderJob,
    transform::Spatial, Frame, Transform,
};

/// Part of a deferred render pipeline
#[derive(Default)]
pub struct GeomPass {
    nodes: FrustumCulled<RenderBundle>,
}

impl GeomPass {
    /// Add a renderbundle to this render pass
    pub fn push<T: Bundle + Spatial>(&mut self, r: &T) {
        self.nodes
            .push(r.bundle(), r.transform().model(), r.bound());
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
        rpass.execute_bundles(self.nodes.cull(Transform::frustum()));
    }
}
