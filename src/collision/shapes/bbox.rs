use glam::Vec3A;

#[derive(Copy, Clone, Debug)]
pub struct BoundingBox {
    pub min: Vec3A,
    pub max: Vec3A,
}
