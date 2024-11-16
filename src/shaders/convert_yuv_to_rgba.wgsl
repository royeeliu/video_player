struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_coords: vec2f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
};

const bt709_limited_yuv_to_rgb = mat3x3f(
    1.164383,  1.164383,  1.164383,
    0.000000, -0.213249,  2.112402,
    1.792741, -0.532909,  0.000000,
);

const bt709_full_yuv_to_rgb = mat3x3f(
    1.000000,  1.000000,  1.000000,
    0.000000, -0.187324,  1.855600,
    1.574800, -0.468124,  0.000000,
);

const bt601_limited_yuv_to_rgb = mat3x3f(
    1.164383,  1.164383,  1.164383,
    0.000000, -0.391762,  2.017232,
    1.596027, -0.812968,  0.000000,
);

const bt601_full_yuv_to_rgb = mat3x3f(
    1.000000,  1.000000,  1.000000,
    0.000000, -0.343730,  1.765380,
    1.402030, -0.714480,  0.000000,
);

const bt2020_limited_yuv_to_rgb = mat3x3f(
    1.164383,  1.164383,  1.164383,
    0.000000, -0.225614,  2.119193,
    1.792705, -0.678001,  0.000000,
);

const bt2020_full_yuv_to_rgb = mat3x3f(
    1.000000,  1.000000,  1.000000,
    0.000000, -0.187326,  1.855600,
    1.574803, -0.468124,  0.000000,
);

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = in.tex_coords;
    out.clip_position = vec4f(in.position, 1.0);
    return out;
}

@group(0) @binding(0)
var t_y: texture_2d<f32>;
@group(0) @binding(1)
var t_u: texture_2d<f32>;
@group(0) @binding(2)
var t_v: texture_2d<f32>;
@group(0) @binding(3)
var s_yuv: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let y = textureSample(t_y, s_yuv, in.tex_coords).r - 16.0 / 255.0;
    let u = textureSample(t_u, s_yuv, in.tex_coords).r - 128.0 / 255.0;
    let v = textureSample(t_v, s_yuv, in.tex_coords).r - 128.0 / 255.0;
    // TODO: 根据实际情况选择转换矩阵
    let rgb = bt709_limited_yuv_to_rgb * vec3f(y, u, v);
    return vec4f(rgb, 1.0);
}