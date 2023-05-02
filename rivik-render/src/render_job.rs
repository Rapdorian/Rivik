/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

#![allow(missing_docs)]

use crate::Frame;

#[derive(Eq, PartialEq, Debug)]
pub struct RenderComponentHandle {
    pass: u64,
    index: usize,
}

pub trait RenderJob {
    fn inputs(&self) -> Vec<&dyn RenderJob> {
        Vec::new()
    }

    fn begin<'a>(&self, frame: &'a mut Frame) -> wgpu::RenderPass<'a>;
    fn render<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>);

    fn run(&self, frame: &mut Frame) {
        for input in self.inputs() {
            input.run(frame)
        }

        // create render pass
        let mut rpass = self.begin(frame);
        self.render(&mut rpass);
    }
}
