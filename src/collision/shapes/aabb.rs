use glam::Vec3A;

#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub min: Vec3A,
    pub max: Vec3A,
}
