// Render pipeline 1.

struct RenderVertexUniform {
    logical_size: vec2<f32>,
};
@group(1) @binding(0)
var<uniform> render_vertex_uniform: RenderVertexUniform;

struct RenderVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct RenderVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(
    model: RenderVertexInput,
) -> RenderVertexOutput {
    var out: RenderVertexOutput;
    out.tex_coords = model.tex_coords;
    out.color = model.color;

    var x: f32 = model.position.x / render_vertex_uniform.logical_size.x;
    var y: f32 = model.position.y / render_vertex_uniform.logical_size.y;

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
fn fs_main(in: RenderVertexOutput) -> @location(0) vec4<f32> {
    let col: vec4<f32> = in.color;
    if col.a > 0.0 {
        return col;
    } else {
        return textureSample(t_diffuse, s_diffuse, in.tex_coords);
    }
}

// Render Pipeline 2

struct PostprocessFragmentUniform {
    render_size: vec2<f32>,
};
@group(2) @binding(0)
var<uniform> postprocessing_fragment_uniform: PostprocessFragmentUniform;

struct PostprocessVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct PostprocessVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main2(
    model: PostprocessVertexInput,
) -> PostprocessVertexOutput {
    var out: PostprocessVertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 0.0f, 1.0f);
    return out;
}

fn tube_warp(coord_: vec2<f32>, offset: vec2<f32>) -> vec2<f32> {
    var coord = (coord_ * 2.0) - 1.0;
    coord *= 0.5;

    coord.x *= (1.0 + pow(coord.y / 2.5, 2.0));
    coord.y *= (1.0 + pow(coord.x / 2.5, 2.0));

    coord += offset;
    coord += 0.5;

    return coord;
}

@fragment
fn fs_main2(in: PostprocessVertexOutput) -> @location(0) vec4<f32> {
    // TODO: Put this into uniforms.
    let iOffset = vec2<f32>(0.0, 0.0);

    let uv = ((in.clip_position.xy - iOffset) / postprocessing_fragment_uniform.render_size);
    let uv1 = tube_warp(uv, vec2<f32>(0.0, 0.0));
    let uv2 = tube_warp(uv, vec2<f32>(0.002, 0.0));
    let uv3 = tube_warp(uv, vec2<f32>(-0.002, 0.0));

    if (uv1.x < 0.0 || uv1.y < 0.0 || uv1.x > 1.0 || uv1.y > 1.0) {
         return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
