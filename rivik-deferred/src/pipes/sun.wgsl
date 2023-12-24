struct VertexOutput{
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) light_color: vec3<f32>,
    @location(2) light_dir: vec3<f32>,
}

struct LightData {
    color: vec4<f32>,
    dir: vec4<f32>,
}

struct LightBuffer {
    inv_proj: mat4x4<f32>,
    lights: array<LightData>,
}

@group(2)
@binding(0)
var<storage> light: LightBuffer;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32, @builtin(instance_index) light_idx: u32) -> VertexOutput {
    var out: VertexOutput;
    let i = i32(in_vertex_index) + 1;
    let x = f32(1 - (i & 2));
    let y = f32(1 - ((i & 1) * 2));

    out.pos = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x+1.0)/2.0, 1.0-(y+1.0)/2.0);
    out.light_color = light.lights[light_idx].color.xyz;
    out.light_dir = light.lights[light_idx].dir.xyz;
    return out;
}

@group(0)
@binding(0)
var samplr: sampler;

@group(1)
@binding(0)
var g_colr: texture_2d<f32>;

@group(1)
@binding(1)
var g_norm: texture_2d<f32>;

@group(1)
@binding(2)
var g_matl: texture_2d<f32>;

@group(1)
@binding(3)
var g_depth: texture_depth_2d;

fn norm(norm: vec2<f32>) -> vec3<f32> {
    // generate z component
    let z = 1.0 - (norm.x*norm.x) - (norm.y*norm.y);
    return vec3<f32>(norm.x, norm.y, z);
}

fn view_space_from_depth(coord: vec2<f32>, depth: f32) -> vec3<f32> {
    let screen = coord * 2.0 - 1.0;
    let clip = vec4<f32>(screen.x, -screen.y, depth, 1.0);
    let view = light.inv_proj * clip;
    return view.xyz / view.w;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let norm = norm(textureSample(g_norm, samplr, in.uv).xy);
    let depth = textureSample(g_depth, samplr, in.uv);
    let pos = view_space_from_depth(in.uv, depth);
    let mat = textureSample(g_matl, samplr, in.uv);
    let col = textureSample(g_colr, samplr, in.uv);


    let light_dir = normalize(in.light_dir.xyz);
    var power = max(dot(norm, light_dir), 0.0);

    let refl_dir = reflect(-light_dir, norm);
    var spec = mat.y * pow(dot(normalize(-pos), refl_dir), 32.0);

    //return vec4<f32>(in.light_dir, 0.0);
    //return vec4<f32>(norm, 0.0);
    //return vec4<f32>(pos, 0.0);
    //return vec4<f32>(in.uv, depth, 0.0);
    //return vec4<f32>(power * in.light_color, 0.0);
    //return vec4<f32>(spec * in.light_color, 0.0);
    //return vec4<f32>((spec + power) * in.light_color, 0.0);
    return vec4<f32>(col.xyz * ((power + spec) * in.light_color), 0.0);
}
