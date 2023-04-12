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
var hdr: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let col = textureSample(hdr, samplr, in.uv);
    return col;
}
