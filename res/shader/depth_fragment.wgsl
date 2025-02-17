struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
}

@group(1) @binding(0)
var t_depth: texture_2d<f32>;
@group(1) @binding(1)
var s_depth: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_depth, s_depth, in.tex_coords).x;
    return vec4<f32>(vec3<f32>(color), 1.0);
}