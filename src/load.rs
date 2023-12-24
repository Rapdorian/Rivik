use std::{fs, path::Path};

use legion::{storage::Component, system, systems::CommandBuffer, Entity};
use rivik_deferred::{
    types::{mesh::MeshPtr, tex::DynTex, Model, Renderer},
    Render,
};

/// Load a TGA texture
fn load_tga(rend: &mut Render, tex: &[u8]) -> u32 {
    let img = image::load_from_memory_with_format(tex, image::ImageFormat::Tga).unwrap();

    let tex = DynTex {
        width: img.width() as u16,
        height: img.height() as u16,
        data: img.to_rgba8().to_vec(),
    };
    rend.upload_texture(None, &tex)
}

#[system(for_each)]
pub fn load_model<P: AsRef<Path> + Component + std::fmt::Debug>(
    entity: &Entity,
    cmd: &mut CommandBuffer,
    model: &mut Model<P>,
    loaded: Option<&Model<u32>>,
    #[resource] render: &mut Render,
) {
    if loaded.is_some() {
        // don't load this model
        return;
    }
    println!("Loading model: {model:#?}");
    // create a new model
    let loaded = Model {
        mesh: render.upload_mesh(None, &MeshPtr::new(&fs::read(&model.mesh).unwrap())),
        diffuse: load_tga(render, &fs::read(&model.diffuse).unwrap()),
        metal: load_tga(render, &fs::read(&model.metal).unwrap()),
        rough: load_tga(render, &fs::read(&model.rough).unwrap()),
        normal: load_tga(render, &fs::read(&model.normal).unwrap()),
    };

    cmd.add_component(*entity, loaded);
    //cmd.remove_component::<Model<String>>(*entity);
}
