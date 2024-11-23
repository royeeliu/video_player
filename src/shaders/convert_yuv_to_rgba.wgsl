struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_coords: vec2f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
};


// https://learn.microsoft.com/en-us/windows/win32/medfound/recommended-8-bit-yuv-formats-for-video-rendering
// the formulas to convert YUV to RGB can be derived as follows
const bt601_limited_yuv_to_rgb = mat3x3f(
    1.164384,  1.164384,  1.164384,
    0.000000, -0.391762,  2.017232,
    1.596027, -0.812968,  0.000000,
);

// https://learn.microsoft.com/en-us/windows/win32/api/dxva2api/ne-dxva2api-dxva2_videotransfermatrix
// BT.601 transfer matrices
// https://en.wikipedia.org/wiki/YCbCr
// JPEG conversion
const bt601_full_yuv_to_rgb = mat3x3f(
    1.000000,  1.000000,  1.000000,
    0.000000, -0.344136,  1.772000,
    1.402000, -0.714136,  0.000000,
);

// https://skia.googlesource.com/skia/+/c9c86fef729d/src/core/SkYUVMath.cpp
const bt709_limited_yuv_to_rgb = mat3x3f(
    1.164384,  1.164384,  1.164384,
    0.000000, -0.213249,  2.112402,
    1.792741, -0.532909,  0.000000,
);

// https://learn.microsoft.com/en-us/windows/win32/api/dxva2api/ne-dxva2api-dxva2_videotransfermatrix
// BT.709 transfer matrices:
const bt709_full_yuv_to_rgb = mat3x3f(
    1.000000,  1.000000,  1.000000,
    0.000000, -0.187324,  1.855600,
    1.574800, -0.468124,  0.000000,
);

// https://skia.googlesource.com/skia/+/c9c86fef729d/src/core/SkYUVMath.cpp
const bt2020_limited_yuv_to_rgb = mat3x3f(
    1.164384,  1.164384,  1.164384,
    0.000000, -0.187326,  2.141772,
    1.678674, -0.650424,  0.000000,
);

// https://skia.googlesource.com/skia/+/c9c86fef729d/src/core/SkYUVMath.cpp
const bt2020_full_yuv_to_rgb = mat3x3f(
    1.000000,  1.000000,  1.000000,
    0.000000, -0.164553,  1.881400,
    1.474600, -0.571353,  0.000000,
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
@group(0) @binding(4)
var<uniform> matrix_index: u32;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    var y = textureSample(t_y, s_yuv, in.tex_coords).r;
    let u = textureSample(t_u, s_yuv, in.tex_coords).r - 128.0 / 255.0;
    let v = textureSample(t_v, s_yuv, in.tex_coords).r - 128.0 / 255.0;

    var matrix: mat3x3f;
    if (matrix_index == 0u) {
        y -= 16.0 / 255.0;
        matrix = bt601_limited_yuv_to_rgb;
    } else if (matrix_index == 1u) {
        matrix = bt601_full_yuv_to_rgb;
    } else if (matrix_index == 2u) {
        y -= 16.0 / 255.0;
        matrix = bt709_limited_yuv_to_rgb;
    } else if (matrix_index == 3u) {
        matrix = bt709_full_yuv_to_rgb;
    } else if (matrix_index == 4u) {
        y -= 16.0 / 255.0;
        matrix = bt2020_limited_yuv_to_rgb;
    } else if (matrix_index == 5u) {
        matrix = bt2020_full_yuv_to_rgb;
    } else {
        y -= 16.0 / 255.0;
        matrix = bt709_limited_yuv_to_rgb;
    }

    let rgb = matrix * vec3f(y, u, v);
    return vec4f(rgb, 1.0);
}