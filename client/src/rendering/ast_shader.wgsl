struct VertexInput {
    @location(0) clip_position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(6) img_index: i32,
}

struct InstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(6) img_index: i32,
}

// A uniform that handles all the uniform data
struct OverallUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> overal_uniform: OverallUniform;

@group(0) @binding(1)
var ast_textures: texture_2d_array<f32>;

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
    out.tex_coords = model.tex_coords;
    out.clip_position = overal_uniform.view_proj * model_matrix * vec4<f32>(model.clip_position, 0., 1.);
    out.img_index = instance.img_index;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //return vec4<f32>(in.img_index, 0., 1., 1.);

    return textureSample(ast_textures, texture_sampler, in.tex_coords, in.img_index);
}