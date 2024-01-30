// Render Vertex

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

// Render Fragment

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

struct PostprocessFragmentUniform {
    render_size: vec2<f32>,
    texture_size: vec2<f32>,
    time_ms: f32,
};
@group(2) @binding(0)
var<uniform> postprocessing_fragment_uniform: PostprocessFragmentUniform;

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
    y += postprocessing_fragment_uniform.time_ms * 5.0;
    y /= 1.5;
    let scanline_mag = sin(y);
    let scanline_color = vec3<f32>(scanline_mag, scanline_mag, scanline_mag);
    return vec4<f32>(scanline_color, 1.0);
}

// Like textureSample, but fuzzes partway between linear and nearest.
fn sample_texture(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, fuzz_sample_uv(uv));
}

fn get_scene_pixel(uv: vec2<f32>) -> vec4<f32> {
    // vec4 spot = spotlight(uv);

    let player_color = sample_texture(uv);
    //player_color = vec4(mix(player_color.rgb, spot.rgb, spot.a), 1.0);

    //vec4 hud_color = sample_texture(iHudTexture, uv);
    //vec4 color = vec4(mix(hud_color.rgb, player_color.rgb, 1.0 - hud_color.a), 1.0);

    return player_color;
}

@fragment
fn fs_main2(in: PostprocessVertexOutput) -> @location(0) vec4<f32> {
    // TODO: Put this into postprocessing_fragment_uniform.
    let iOffset = vec2<f32>(0.0, 0.0);

    let uv = ((in.clip_position.xy - iOffset) / postprocessing_fragment_uniform.render_size);
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
    color = mix(color, scan, 0.015);

    return color;
}
