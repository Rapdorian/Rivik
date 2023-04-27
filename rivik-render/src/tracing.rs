/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! Draw tracing spans to an EGUI component

use std::{
    collections::{hash_map::DefaultHasher, BTreeMap},
    hash::Hasher,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
    time::Instant,
};

use egui::{
    plot::{Bar, BarChart, PlotUi},
    Color32,
};
use once_cell::sync::Lazy;
use palette::{FromColor, Srgb};
use tracing::{event, field::Visit, span::Attributes, Id, Level, Subscriber};
use tracing_core::Field;
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

#[derive(Clone)]
struct SpanBar {
    name: String,
    fields: Vec<String>,
    depth: usize,
    start: Instant,
    end: Option<Instant>,
}

impl Visit for SpanBar {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.fields.push(format!("{field}: {value:#?}"));
    }
}

struct Event {
    name: String,
    time: Instant,
    depth: usize,
    level: Level,
}

static SPAN_TREE: Lazy<Arc<RwLock<BTreeMap<u64, SpanBar>>>> =
    Lazy::new(|| Arc::new(RwLock::new(BTreeMap::new())));

static FRAME_SPANS: RwLock<Vec<SpanBar>> = RwLock::new(Vec::new());
static FRAME_EVENTS: RwLock<Vec<Event>> = RwLock::new(Vec::new());
static SPAN_DEPTH: AtomicUsize = AtomicUsize::new(0);
static FRAME_START: Lazy<Arc<RwLock<Instant>>> =
    Lazy::new(|| Arc::new(RwLock::new(Instant::now())));

/// Display a record of a frame's span trace to an EGUI plot
#[tracing::instrument(skip(plot, spans, events))]
pub fn display_traces(plot: &mut PlotUi, spans: Vec<Bar>, events: Vec<Bar>) {
    plot.bar_chart(
        BarChart::new(events.clone())
            .highlight(true)
            .name("Events")
            .width(0.01),
    );

    plot.bar_chart(
        BarChart::new(vec![Bar::new(-1.0, 1_000.0 / 60.0)
            .name("Frame Budget")
            .fill(Color32::BLACK)])
        .vertical()
        .horizontal()
        .width(1.0),
    );

    plot.bar_chart(
        BarChart::new(spans)
            .highlight(true)
            .horizontal()
            .width(1.0)
            .name("Spans"),
    );

    plot.bar_chart(
        BarChart::new(events)
            .highlight(true)
            .name("Events")
            .width(0.01),
    );
}

#[tracing::instrument]
fn generate_events() -> Vec<Bar> {
    let mut chart = Vec::new();

    for event in &*FRAME_EVENTS.read().unwrap() {
        let time = (event.time - *FRAME_START.read().unwrap()).as_secs_f64() * 1_000.0;

        // create a color for this span
        let color = match event.level {
            Level::WARN => Color32::YELLOW,
            Level::INFO => Color32::GREEN,
            Level::ERROR => Color32::RED,
            Level::DEBUG => Color32::WHITE,
            Level::TRACE => Color32::GRAY,
        };

        chart.push(
            Bar::new(time, event.depth as f64 + 1.0)
                .name(&event.name)
                .base_offset(-1.5)
                //.stroke(Stroke::new(1.0, Color32::BLACK))
                .fill(color),
        );
    }

    FRAME_EVENTS.write().unwrap().clear();
    chart
}

/// Record a frame's span trace
pub fn generate_chart() -> (Vec<Bar>, Vec<Bar>) {
    let mut chart = Vec::new();

    for span in &*FRAME_SPANS.read().unwrap() {
        let end = match span.end {
            Some(e) => e,
            None => panic!("This shouldn't happen"),
        };

        // create a color for this span
        let mut hasher = DefaultHasher::new();
        hasher.write(span.name.as_bytes());
        let color = hasher.finish();

        let hue = (color & 0xFFFF) as f32 / (0xFFFF as f32);
        let sat = (color >> 8 & 0xFFFF) as f32 / (0xFFFF as f32);
        let light = (color >> 16 & 0xFFFF) as f32 / (0xFFFF as f32);
        let light = light.max(0.4);
        let sat = sat.max(0.7);

        let color = palette::Hsl::new(hue * 360.0, sat, light);
        let color = Srgb::from_color(color);
        let r = (color.red * 256.0) as u8;
        let g = (color.green * 256.0) as u8;
        let b = (color.blue * 256.0) as u8;

        // generate a bar for this span
        chart.push(
            Bar::new(
                span.depth as f64,
                (end - span.start).as_secs_f64().min(1.0 / 60.0) * 1_000.0,
            )
            .base_offset((span.start - *FRAME_START.read().unwrap()).as_secs_f64() * 1_000.0)
            .name(&format!("{} {}", span.name, span.fields.join("\n")))
            .fill(Color32::from_rgb(r, g, b)),
        );
    }

    let events = generate_events();
    // reset frame
    {
        FRAME_SPANS.write().unwrap().clear();
        *FRAME_START.write().unwrap() = Instant::now();
    }
    (chart, events)
}
//}

/// A tracing subscriber that prepares traces for drawing to an EGUI plot
#[derive(Default)]
pub struct UiSubscriber {}

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for UiSubscriber {
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let time = Instant::now();
        let name = ctx.metadata(id).unwrap().name();

        let mut bar = SpanBar {
            name: name.to_owned(),
            fields: Vec::new(),
            depth: 0,
            start: time,
            end: None,
        };
        attrs.values().record(&mut bar);
        let _ = SPAN_TREE.write().unwrap().insert(id.into_u64(), bar);
    }

    fn on_enter(&self, id: &Id, _ctx: Context<'_, S>) {
        let depth = SPAN_DEPTH.fetch_add(1, Ordering::Relaxed);
        let mut tree = SPAN_TREE.write().unwrap();
        let span = tree.get_mut(&id.into_u64()).unwrap();
        span.start = Instant::now();
        span.depth = depth;
    }

    fn on_exit(&self, id: &Id, _: Context<'_, S>) {
        let depth = SPAN_DEPTH.fetch_sub(1, Ordering::Relaxed);
        let time = Instant::now();

        // get this span out of the tree
        let mut span = SPAN_TREE
            .write()
            .unwrap()
            .get(&id.into_u64())
            .expect("span should be in tree")
            .clone();
        span.end = Some(time);
        assert_eq!(depth, span.depth + 1);
        FRAME_SPANS.write().unwrap().push(span);
    }

    fn on_close(&self, id: Id, _: Context<'_, S>) {
        let _ = SPAN_TREE.write().unwrap().remove(&id.into_u64()).unwrap();
    }

    fn on_event(&self, event: &event::Event<'_>, _ctx: Context<'_, S>) {
        let time = Instant::now();

        #[derive(Default)]
        struct EventRecorder {
            name: String,
            fields: Vec<String>,
        }

        impl Visit for EventRecorder {
            fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.name = format!("{value:#?}");
                } else {
                    self.fields.push(format!("{value:#?}"));
                }
            }
        }
        let mut name = EventRecorder::default();

        event.record(&mut name);

        let depth = SPAN_DEPTH.load(Ordering::Relaxed);
        let level = event.metadata().level().clone();
        FRAME_EVENTS.write().unwrap().push(Event {
            name: name.name,
            time,
            depth,
            level,
        });
    }
}
