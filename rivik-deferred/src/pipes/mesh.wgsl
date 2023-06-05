struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) view_position: vec4<f32>,
    @location(2) norm: vec4<f32>,
}

@group(0)
@binding(0)
var samplr: sampler;

struct Transform {
    mvp: mat4x4<f32>,
    mv: mat4x4<f32>,
    mv_norm: mat4x4<f32>,
}

@group(1)
@binding(0)
var<uniform> transform: Transform;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) norm: vec3<f32>,
    @location(2) uv: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = transform.mvp * vec4<f32>(position, 1.0);
    out.uv = uv;
    out.view_position = transform.mv * vec4<f32>(position, 1.0);
    out.norm = normalize(transform.mv_norm * vec4<f32>(norm, 0.0));
    return out;
}

@group(2)
@binding(0)
var m_colr: texture_2d<f32>;

@group(2)
@binding(1)
var m_rough: texture_2d<f32>;

@group(2)
@binding(2)
var m_metal: texture_2d<f32>;

@group(2)
@binding(3)
var m_norm: texture_2d<f32>;

struct GBuffer {
    @location(0)
    color: vec4<f32>,
    @location(1)
    normal: vec2<f32>,
    @location(2)
    material: vec4<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> GBuffer {
    var gbuffer: GBuffer;

    gbuffer.color    = textureSample(m_colr, samplr, in.uv);
    let norm_tex  = normalize(textureSample(m_norm, samplr, in.uv).xyz).xyz;
    gbuffer.normal = normalize(in.norm.xyz + normalize(norm_tex)).xy;
    let rough = textureSample(m_rough, samplr, in.uv);
    let metal = textureSample(m_metal, samplr, in.uv);
    gbuffer.material = vec4<f32>(rough.x, metal.x, 0.0, 0.0);

    return gbuffer;
}
