struct VertexOutput{
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let i = i32(in_vertex_index) + 1;
    let x = f32(1 - (i & 2));
    let y = f32(1 - ((i & 1) * 2));

    out.pos = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x+1.0)/2.0, 1.0-(y+1.0)/2.0);
    return out;
}

@group(0)
@binding(0)
var samplr: sampler;

@group(0)
@binding(1)
var g_color: texture_2d<f32>;

@group(0)
@binding(2)
var g_pos: texture_2d<f32>;

@group(0)
@binding(3)
var g_norm: texture_2d<f32>;

struct LightData{
    color: vec4<f32>,
    direction: vec4<f32>,
}

@group(1)
@binding(0)
var<uniform> light_data: LightData;

fn sq_len(v: vec3<f32>) -> f32 {
    return v.x * v.x + v.y * v.y;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let norm = textureSample(g_norm, samplr, in.uv).xyz;
    let col = textureSample(g_color, samplr, in.uv);
    let pos = textureSample(g_pos, samplr, in.uv).xyz;

    let light_dir = light_data.direction.xyz;
    let power = max(dot(norm, normalize(light_dir)), 0.0);

    // TODO: Specular strength/intensity should be in the gbuffer
    let spec_str = 0.5;
    let shininess = 16.0;

    let refl_dir = reflect(-light_dir, norm);
    let spec = spec_str * pow(clamp(dot(normalize(-pos), refl_dir), 0.0, 1.0), shininess);

    let power = round(power * 4.) / 4.;
    let spec = round(spec* 4.) / 4.;

    // let towardsLight = dot(norm, normalize(light_dir));
    // let lightIntensity = step(0., towardsLight);

    return col * (power + spec);
}
