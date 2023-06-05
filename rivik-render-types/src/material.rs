use crate::tex::DynTex;

#[derive(Clone, Debug)]
pub struct Material {
    pub diffuse: DynTex,
    /// Use as spec exponent when phong shading
    pub rough: DynTex,
    /// Use as spec power when phong shading
    pub metal: DynTex,
    pub normal: DynTex,
}
