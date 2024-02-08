// Render Vertex

struct RenderVertexUniform {
    logical_size: vec2<f32>,
    unused: vec2<f32>,
};
@group(0) @binding(0)
var<uniform> render_vertex_uniform: RenderVertexUniform;

struct DefaultUniform {
    unused: f32,
}
@group(1) @binding(0)
var<uniform> default_uniform: DefaultUniform;

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

// Render Fragment

@group(2) @binding(0)
var texture_atlas: texture_2d<f32>;
@group(2) @binding(1)
var texture_atlas_sampler: sampler;

@fragment
fn fs_main(in: RenderVertexOutput) -> @location(0) vec4<f32> {
    let col: vec4<f32> = in.color;
    if col.a > 0.0 {
        return col;
    } else {
        return textureSample(texture_atlas, texture_atlas_sampler, in.tex_coords);
    }
}

// Postprocessing Vertex

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

// Postprocessing Fragment

struct Light {
    position: vec2<f32>,
    radius: f32,
    padding: f32,
}

struct PostprocessFragmentUniform {
    render_size: vec2<f32>,
    texture_size: vec2<f32>,
    time_s: f32,

    // Lighting
    is_dark: i32,
    spotlight_count: i32,
    spotlight: array<Light, 32>,
};
@group(1) @binding(0)
var<uniform> postprocessing_fragment_uniform: PostprocessFragmentUniform;

@group(2) @binding(0)
var player_framebuffer_texture: texture_2d<f32>;
@group(2) @binding(1)
var player_framebuffer_sampler: sampler;

@group(2) @binding(2)
var hud_framebuffer_texture: texture_2d<f32>;
@group(2) @binding(3)
var hud_framebuffer_sampler: sampler;

@group(2) @binding(4)
var static_texture: texture_2d<f32>;
@group(2) @binding(5)
var static_sampler: sampler;

fn spotlight(position_: vec2<f32>) -> vec4<f32> {
    var position = position_;

    if (postprocessing_fragment_uniform.is_dark == 0) {
        return vec4<f32>(1.0, 1.0, 1.0, 0.0);
    }
    if (postprocessing_fragment_uniform.spotlight_count == 0) {
        return vec4<f32>(1.0, 1.0, 1.0, 0.0);
    }
    //position.y = 1.0 - position.y;
    position *= postprocessing_fragment_uniform.texture_size;

    var alpha: f32 = 1.0;
    for (var i = 0; i < postprocessing_fragment_uniform.spotlight_count; i++) {
        let spotlight_position = postprocessing_fragment_uniform.spotlight[i].position;
        let d = distance(spotlight_position, position);
        let a = smoothstep(0.0, 1.0, d / postprocessing_fragment_uniform.spotlight[i].radius) * 0.85;
        alpha = min(alpha, a);
    }

    return vec4<f32>(0.0, 0.0, 0.0, alpha);
}

// This is like, halfway between LINEAR and NEAREST.
// It requires the texture be sampled with LINEAR.
fn fuzz_sample_uv(coord_: vec2<f32>) -> vec2<f32>{
    var coord = coord_;
    coord *= postprocessing_fragment_uniform.texture_size;
    coord += 0.5;

    var coordFloor = floor(coord) + 0.5;
    var coordFract = fract(coord);

    coordFract.x = smoothstep(-0.5, 0.5, coordFract.x - 0.5);
    coordFract.y = smoothstep(-0.5, 0.5, coordFract.y - 0.5);
    // coordFract = vec2(0.0, 0.0);

    coord = coordFloor + coordFract;
    coord /= postprocessing_fragment_uniform.texture_size;
    return coord;
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

fn scanline(y_: f32) -> vec4<f32> {
    var y = y_;
    y *= postprocessing_fragment_uniform.render_size.y;
    y += postprocessing_fragment_uniform.time_s * 5.0;
    y /= 1.5;
    let scanline_mag = sin(y);
    let scanline_color = vec3<f32>(scanline_mag, scanline_mag, scanline_mag);
    return vec4<f32>(scanline_color, 1.0);
}

fn get_scene_pixel(uv: vec2<f32>) -> vec4<f32> {
    let spot = spotlight(uv);

    let fuzzed_sample_uv = fuzz_sample_uv(uv);

    var player_color = textureSample(player_framebuffer_texture, player_framebuffer_sampler, fuzzed_sample_uv);
    player_color = vec4(mix(player_color.rgb, spot.rgb, spot.a), 1.0);

    let hud_color = textureSample(hud_framebuffer_texture, hud_framebuffer_sampler, fuzzed_sample_uv);
    let color = vec4<f32>(mix(hud_color.rgb, player_color.rgb, 1.0 - hud_color.a), 1.0);

    return color;
}

@fragment
fn fs_main2(in: PostprocessVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.clip_position.xy / postprocessing_fragment_uniform.render_size;
    let uv1 = tube_warp(uv, vec2<f32>(0.0, 0.0));
    let uv2 = tube_warp(uv, vec2<f32>(0.002, 0.0));
    let uv3 = tube_warp(uv, vec2<f32>(-0.002, 0.0));

    if (uv1.x < 0.0 || uv1.y < 0.0 || uv1.x > 1.0 || uv1.y > 1.0) {
         return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    let color1 = get_scene_pixel(uv1);
    let color2 = get_scene_pixel(uv2);
    let color3 = get_scene_pixel(uv3);
    var color = vec4<f32>(color2.r, color1.g, color3.b, 1.0);

    let scan = scanline(uv1.y);

    var random_pos = uv1;
    random_pos.y += postprocessing_fragment_uniform.time_s * 10.0;
    random_pos = modf(random_pos).fract;
    let random = textureSample(static_texture, static_sampler, random_pos);

    color = mix(mix(color, random, 0.04), scan, 0.015);

    return color;
}
