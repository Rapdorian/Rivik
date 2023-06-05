use toml_scene::Scene;

fn main() {
    let a: Scene = toml::from_str(include_str!("scene.toml")).unwrap();
    println!("{a:#?}");
}
