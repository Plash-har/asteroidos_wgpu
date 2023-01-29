struct VertexInput {
    @location(0) clip_position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(6) accent_color_0: vec4<f32>,
    @location(7) accent_color_1: vec4<f32>,
    @location(8) accent_color_2: vec4<f32>,
    @location(9) accent_color_3: vec4<f32>,
    @location(10) accent_flame_color: vec4<f32>,
    @location(11) flame_frame: f32,
    @location(12) player_idx: i32,
}

struct InstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(6) accent_color_0: vec4<f32>,
    @location(7) accent_color_1: vec4<f32>,
    @location(8) accent_color_2: vec4<f32>,
    @location(9) accent_color_3: vec4<f32>,
    @location(10) accent_flame_color: vec4<f32>,
    @location(11) flame_frame: f32,
    @location(12) player_idx: i32,
}

// A uniform that handles all the uniform data
struct OverallUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> overal_uniform: OverallUniform;

@group(0) @binding(1)
var player_textures: texture_2d_array<f32>;

@group(0) @binding(2)
var texture_sampler: sampler;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3
    );

    var out: VertexOutput;
    out.clip_position = overal_uniform.view_proj * model_matrix * vec4<f32>(model.clip_position, 0., 1.);
    
    out.accent_color_0 = instance.accent_color_0;
    out.accent_color_1 = instance.accent_color_1;
    out.accent_color_2 = instance.accent_color_2;
    out.accent_color_3 = instance.accent_color_3;
    out.accent_flame_color = instance.accent_flame_color;
    out.flame_frame = instance.flame_frame;
    out.player_idx = instance.player_idx;

    if model.tex_coords.x > 0.5 {
        out.tex_coords = vec2<f32>(model.tex_coords.x, model.tex_coords.y + instance.flame_frame);
    } else {
        out.tex_coords = model.tex_coords;
    }

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSample(player_textures, texture_sampler, in.tex_coords, in.player_idx);

    // let sample = vec4<f32>(in.tex_coords, 0., 1.);

    if in.tex_coords.x > 0.5 {
        if in.flame_frame < -1. { // Flammes désactivée
            return vec4<f32>(0., 0., 0., 0.);
        }

        return sample * in.accent_flame_color;
    }

    if in.tex_coords.y < 0.25 {
        return sample * in.accent_color_0;
    } 
    if in.tex_coords.y < 0.5 {
        return sample * in.accent_color_1;
    }
    if in.tex_coords.y < 0.75 {
        return sample * in.accent_color_2;
    }

    return sample * in.accent_color_3;

    // return vec4<f32>(1., 1., 1., 1.,);
}