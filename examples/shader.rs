use rivik_shader::prims::{Vec3, F32};

struct Output {
    foo: F32,
    bar: Vec3,
}

fn shader(i: F32, b: F32) -> Output {
    Output {
        foo: i.clone() * 3.0,
        bar: Vec3::new(i.clone(), b.clone(), (i + b) / 3.0).normalize(),
    }
}

fn main() {
    let Output { foo, bar } = shader(F32::bind("i"), F32::bind("b"));
    println!(
        r#"
struct Output {{
    foo: f32,
    bar: f32,
}}

fs_main(i: f32, b: f32) -> Output {{
    var out: Output;
    out.foo = {foo};
    out.bar = {bar};
    return out;
}}"#,
    );
}
