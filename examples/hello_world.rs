/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

use rivik::{
    egui::{self, CentralPanel},
    run,
};
use rivik_render::tracing::UiSubscriber;
use tracing::{dispatcher::set_global_default, Dispatch};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, Registry};

#[derive(Default)]
pub struct App {}

impl rivik::App for App {
    fn ui(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello World");
        });
    }

    fn init(ctx: &mut rivik::Context) -> Self {
        ctx.show_trace = true;
        Self {}
    }
}

fn main() {
    set_global_default(Dispatch::new(
        Registry::default().with(UiSubscriber::default()),
    ))
    .unwrap();
    run::<App>();
}
