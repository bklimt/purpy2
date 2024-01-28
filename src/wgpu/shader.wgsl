struct ShaderUniform {
    logical_size: vec2<f32>,
};
@group(1) @binding(0)
var<uniform> uniforms: ShaderUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.color = model.color;

    var x: f32 = model.position.x / uniforms.logical_size.x;
    var y: f32 = model.position.y / uniforms.logical_size.y;

    out.clip_position =  vec4<f32>(
        x * 2.0 - 1.0,
        (1.0 - y) * 2.0 - 1.0,
        0.0,
        1.0);

    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let col: vec4<f32> = in.color;
    if col.a > 0.0 {
        return col;
    } else {
        return textureSample(t_diffuse, s_diffuse, in.tex_coords);
    }
}

struct VertexInput2 {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput2 {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main2(
    model: VertexInput2,
) -> VertexOutput2 {
    var out: VertexOutput2;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 0.0f, 1.0f);
    return out;
}

@fragment
fn fs_main2(in: VertexOutput2) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
