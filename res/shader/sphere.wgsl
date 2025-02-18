struct CameraUniform {
    view_pos: vec4<f32>,
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
    @location(9) normal_matrix_0: vec3<f32>,
    @location(10) normal_matrix_1: vec3<f32>,
    @location(11) normal_matrix_2: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
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

    let normal_matrix = mat3x3<f32>(
        transform.normal_matrix_0,
        transform.normal_matrix_1,
        transform.normal_matrix_2,
    );

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;

    var world_position: vec4<f32> = model_matrix * vec4<f32>(model.position, 1.0);
    out.clip_position = camera.view_proj * world_position;
    out.world_normal = normal_matrix * model.normal;
    out.world_position = world_position.xyz;
    return out;
}

struct LightUniform {
    position: vec3<f32>,
    color: vec3<f32>,
}

@group(1) @binding(0)
var<uniform> light: LightUniform;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color = vec4<f32>(in.world_normal, 1.0);

    let ambient_strength = 0.1;
    let ambient_color = light.color * ambient_strength;

    let light_dir = normalize(light.position - in.world_position);

    let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    let view_dir = normalize(camera.view_pos.xyz - in.world_position);
    let half_dir = normalize(view_dir + light_dir);

    let specular_strength = pow(max(dot(view_dir, light_dir), 0.0), 32.0);
    let specular_color = specular_strength * light.color;

    let result = (ambient_color + diffuse_color + specular_color) * object_color.xyz;

    return vec4<f32>(result, object_color.a);
}