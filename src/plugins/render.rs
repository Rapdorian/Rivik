use render::Render;
pub use rivik_deferred as render;

use super::life_cycle::LifeCycle;

// create a render plugin
pub fn renderer(app: &LifeCycle) -> Render {
    Render::new(app.window())
}
