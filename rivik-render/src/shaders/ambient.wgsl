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


@group(0)
@binding(4)
var g_lum: texture_2d<f32>;

struct LightData{
    color: vec4<f32>,
}

@group(1)
@binding(0)
var<uniform> light_data: LightData;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let col = textureSample(g_color, samplr, in.uv);
    let lum = textureSample(g_lum, samplr, in.uv);

    let lum_col = col * lum;
    let light = col * light_data.color;
    return max(light, lum_col);
}
