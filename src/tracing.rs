//! Draw tracing spans to an EGUI component

use std::{
    collections::{hash_map::DefaultHasher, BTreeMap},
    hash::Hasher,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use egui::{plot::Bar, Color32};
use palette::{rgb::Rgb, FromColor, Srgb};
use tracing::{span, Subscriber};

struct SpanBar {
    name: String,
    color: Color32,
    depth: usize,
    start: Instant,
    end: Option<Instant>,
}

pub struct UiSubscriberData {
    spans: BTreeMap<u64, SpanBar>,
    depth: usize,
    frame_start: Instant,
}

impl UiSubscriberData {
    pub fn generate_chart(&mut self) -> Vec<Bar> {
        let mut chart = Vec::new();

        chart.push(
            Bar::new(-1.0, 1_000.0 / 60.0)
                .name("Frame Budget")
                .fill(Color32::BLACK),
        );

        let mut kill_list = Vec::new();
        for (id, span) in &self.spans {
            let end = match span.end {
                Some(e) => e,
                None => self.frame_start + Duration::from_millis(1000 / 60),
            };
            // generate a bar for this span
            chart.push(
                Bar::new(
                    span.depth as f64,
                    (end - span.start).as_secs_f64().min(1.0 / 60.0) * 1_000.0,
                )
                .base_offset((span.start - self.frame_start).as_secs_f64() * 1_000.0)
                .name(&span.name)
                .fill(span.color),
            );
            if span.end.is_some() {
                kill_list.push(*id);
            }
        }
        for dead_guy in kill_list {
            let _ = self.spans.remove(&dead_guy);
        }
        // reset frame
        {
            self.frame_start = Instant::now();
        }
        chart
    }
}

pub struct UiSubscriber {
    data: Arc<RwLock<UiSubscriberData>>,
}

impl UiSubscriber {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(UiSubscriberData {
                spans: BTreeMap::new(),
                depth: 0,
                frame_start: Instant::now(),
            })),
        }
    }

    pub fn data(&self) -> Arc<RwLock<UiSubscriberData>> {
        self.data.clone()
    }
}

impl Subscriber for UiSubscriber {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, span: &span::Attributes<'_>) -> span::Id {
        let mut data = self.data.write().unwrap();

        let mut hasher = DefaultHasher::new();
        hasher.write(span.metadata().name().as_bytes());
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

        let mut name = String::with_capacity(span.metadata().name().len() + 100);
        name.push_str(span.metadata().name());
        if span.values().len() > 0 {
            name.push_str(&span.values().to_string());
        }

        let bar = SpanBar {
            name,
            color: Color32::from_rgb(r, g, b),
            depth: 0,
            start: Instant::now(),
            end: None,
        };
        let id = (data.spans.last_key_value().map(|(k, _)| *k).unwrap_or(0) + 1) as u64;
        let None = data.spans.insert(id, bar) else { panic!("IDK what happened but its not good") };
        span::Id::from_u64(id)
    }

    fn record(&self, span: &span::Id, values: &span::Record<'_>) {}

    fn record_follows_from(&self, span: &span::Id, follows: &span::Id) {}

    fn event(&self, event: &tracing::Event<'_>) {}

    fn enter(&self, span: &span::Id) {
        let mut data = self.data.write().unwrap();

        let depth = data.depth;
        let start = data.frame_start.elapsed();
        // update this span now
        {
            let span = &mut data.spans.entry(span.into_u64()).and_modify(|span| {
                span.depth = depth;
                span.start = Instant::now();
            });
        }
        data.depth += 1;
    }

    fn exit(&self, span: &span::Id) {
        let mut data = self.data.write().unwrap();

        let end = data.frame_start.elapsed();
        // update this span
        {
            let span = &mut data.spans.entry(span.into_u64()).and_modify(|span| {
                span.end = Some(Instant::now());
            });
        }
        data.depth -= 1;
    }
}
