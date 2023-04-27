//  This Source Code Form is subject to the terms of the Mozilla Public License,
//  v. 2.0. If a copy of the MPL was not distributed with this file, You can
//  obtain one at http://mozilla.org/MPL/2.0/.


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
    out.uv = vec2<f32>((x+1.0)/2.0, (y+1.0)/2.0);
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

fn sq_len(v: vec3<f32>) -> f32 {
    return v.x * v.x + v.y * v.y;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let norm = textureSample(g_norm, samplr, in.uv).xyz;
    let col = textureSample(g_color, samplr, in.uv);
    let pos = textureSample(g_pos, samplr, in.uv).xyz;

    // TODO: These sholdn't be constants
    let cam_pos = vec3<f32>(2.5, 2.0, 2.5);

    // get light dir
    // TODO: This shouldn't be hardcoded
    let light_pos = vec3<f32>(2.5, 2.0, 2.5);
    let light_dir = light_pos - pos;
    let power = 3.0 / sq_len(light_dir);
    let spec_str = 2.0 / sq_len(light_dir);
    let power = power * max(dot(norm, normalize(light_dir)), 0.0);

    let view_dir = normalize(cam_pos - pos);
    let refl_dir = reflect(-light_dir, norm);
    let spec = pow(clamp(dot(view_dir, refl_dir), 0.0, 1.0), 16.0);
    let spec = spec * spec_str;
    let white = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    return col * (power + spec);
}
