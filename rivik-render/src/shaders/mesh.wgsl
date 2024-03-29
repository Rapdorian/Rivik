//  This Source Code Form is subject to the terms of the Mozilla Public License,
//  v. 2.0. If a copy of the MPL was not distributed with this file, You can
//  obtain one at http://mozilla.org/MPL/2.0/.

struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @location(1) norm: vec4<f32>,
    @builtin(position) position: vec4<f32>,
    @location(2) view_position: vec4<f32>,
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
    @location(0) position: vec3<f32>,
    @location(1) norm: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = vec2<f32>(tex_coord.x, 1.0 - tex_coord.y);
    out.position = r_locals.mvp * vec4<f32>(position.xyz, 1.0);
    out.view_position = r_locals.mv * vec4<f32>(position.xyz, 1.0);
    out.norm = r_locals.mv_norm* vec4<f32>(norm.xyz, 0.0);
    return out;
}

struct GBuffer {
    @location(0)
    color: vec4<f32>,
    @location(1)
    pos: vec4<f32>,
    @location(2)
    normal: vec4<f32>,
    @location(3)
    lum: vec4<f32>,
}

@group(0)
@binding(0)
var samplr: sampler;

@group(0)
@binding(1)
var g_diffuse: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> GBuffer {
    var gbuffer: GBuffer;

    gbuffer.color = textureSample(g_diffuse, samplr, in.tex_coord);
    gbuffer.pos = in.view_position;
    gbuffer.normal = in.norm;

    return gbuffer;
}
