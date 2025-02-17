struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct TransformInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    transform: TransformInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        transform.model_matrix_0,
        transform.model_matrix_1,
        transform.model_matrix_2,
        transform.model_matrix_3,
    );

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;

    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    out.normal = model.normal;
    return out;
}