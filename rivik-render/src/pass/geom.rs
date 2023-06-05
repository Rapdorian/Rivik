use crate::{context::gbuffer, render_job::RenderPass};

/// Renders geometry to the gbuffer
pub struct GeomPass<'r> {
    rpass: wgpu::RenderPass<'r>,
}

impl<'r> RenderPass<'r> for GeomPass<'r> {
    fn begin(frame: &'r mut crate::Frame) -> Self {
        Self {
            rpass: gbuffer().rpass(
                Some("Deferred Geom Pass"),
                &mut frame.encoder,
                Some(wgpu::Color::BLACK),
            ),
        }
    }
}

impl<'r> GeomPass<'r> {
    // allow issuing cusom draw commands here
}
