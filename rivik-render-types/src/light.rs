/// Describes a light
pub enum Light {
    Ambient {
        color: [f32; 3],
    },
    Directional {
        color: [f32; 3],
        direction: [f32; 3],
    },
    Point {
        color: [f32; 3],
        position: [f32; 3],
    },
}
