//  This Source Code Form is subject to the terms of the Mozilla Public License,
//  v. 2.0. If a copy of the MPL was not distributed with this file, You can
//  obtain one at http://mozilla.org/MPL/2.0/.

struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @location(1) norm: vec4<f32>,
    @builtin(position) position: vec4<f32>,
    @location(2) uv_a: vec2<f32>,
    @location(3) uv_b: vec2<f32>,
    @location(4) uv_c: vec2<f32>,
    @location(5) pos_a: vec4<f32>,
    @location(6) pos_b: vec4<f32>,
    @location(7) pos_c: vec4<f32>,
    @location(8) norm_a: vec3<f32>,
    @location(9) norm_b: vec3<f32>,
    @location(10) norm_c: vec3<f32>,

}

struct Locals {
    mvp: mat4x4<f32>,
    mv: mat4x4<f32>,
    mv_norm: mat4x4<f32>,
}

@group(1)
@binding(0)
var<uniform> r_locals: Locals;

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) norm: vec4<f32>,
    @location(2) tex_coord: vec2<f32>,
    @location(3) uv_a: vec2<f32>,
    @location(4) uv_b: vec2<f32>,
    @location(5) uv_c: vec2<f32>,
    @location(6) pos_a: vec3<f32>,
    @location(7) pos_b: vec3<f32>,
    @location(8) pos_c: vec3<f32>,
    @location(9) norm_a: vec3<f32>,
    @location(10) norm_b: vec3<f32>,
    @location(11) norm_c: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = vec2<f32>(tex_coord.x, 1.0 - tex_coord.y);
    out.position = r_locals.mvp * vec4<f32>(position.xyz, 1.0);
    out.norm = r_locals.mv_norm* vec4<f32>(norm.xyz, 0.0);
    out.uv_a = vec2<f32>(uv_a.x, 1.0 - uv_a.y);
    out.uv_b = vec2<f32>(uv_b.x, 1.0 - uv_b.y);
    out.uv_c = vec2<f32>(uv_c.x, 1.0 - uv_c.y);
    out.pos_a = r_locals.mv * vec4<f32>(pos_a, 1.0);
    out.pos_b = r_locals.mv * vec4<f32>(pos_b, 1.0);
    out.pos_c = r_locals.mv * vec4<f32>(pos_c, 1.0);
    out.norm_a = (r_locals.mv_norm * vec4<f32>(norm_a.xyz, 0.0)).xyz;
    out.norm_b = (r_locals.mv_norm * vec4<f32>(norm_b.xyz, 0.0)).xyz;
    out.norm_c = (r_locals.mv_norm * vec4<f32>(norm_c.xyz, 0.0)).xyz;

    return out;
}

struct GBuffer {
    @location(0)
    color: vec4<f32>,
    @location(1)
    pos: vec4<f32>,
    @location(2)
    normal: vec4<f32>,
}

@group(0)
@binding(0)
var samplr: sampler;

@group(0)
@binding(1)
var g_diffuse: texture_2d<f32>;

fn bary(a: vec2<f32>, b: vec2<f32>, c: vec2<f32>, uv: vec2<f32>) -> vec3<f32> {
    let denom = (b.y-c.y)*(a.x-c.x)+(c.x-b.x)*(a.y-c.y);
    let denom2 =(c.y-a.y)*(b.x-c.x)+(a.x-c.x)*(b.y-c.y);
    let x = ((b.y-c.y)*(uv.x-c.x)+(c.x-b.x)*(uv.y-c.y)) / denom;
    let y = ((c.y-a.y)*(uv.x-c.x)+(a.x-c.x)*(uv.y-c.y)) / denom2;
    let z = 1.0 - x - y;
    return vec3<f32>(x, y, z);
}

@fragment
fn fs_main(in: VertexOutput) -> GBuffer {
    var gbuffer: GBuffer;

    gbuffer.color = textureSample(g_diffuse, samplr, in.tex_coord);
    //gbuffer.color = vec4<f32>(in.tex_coord, 0.0, 1.0);
    //gbuffer.color = vec4<f32>(1.0, 1.0, 1.0, 0.0);
    //gbuffer.color = in.norm;
    //gbuffer.pos = in.position;
    let dim = vec2<f32>(textureDimensions(g_diffuse));
    var f_uv = in.tex_coord;
    f_uv = floor(f_uv * dim) / dim;
    let bary_coord = bary(in.uv_a, in.uv_b, in.uv_c, f_uv);
    let f_pos = in.pos_a.xyz * bary_coord.x + in.pos_b.xyz * bary_coord.y + in.pos_c.xyz * bary_coord.z;

    gbuffer.pos = vec4<f32>(f_pos, 1.0);

    let norm = in.norm_a * bary_coord.x + in.norm_b * bary_coord.y + in.norm_c *
    bary_coord.z;

    gbuffer.normal = vec4<f32>(norm, 0.0);

    return gbuffer;
}
