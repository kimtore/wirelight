/// The following code was first written by GPT-o4,
/// and subsequently modified.

use num_traits::Float;

#[derive(Default, Debug, Clone, Copy)]
struct RGB {
    r: f32,
    g: f32,
    b: f32,
}

#[derive(Debug, Clone, Copy)]
struct XYZ {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Clone, Copy)]
struct CIELUV {
    l: f32,
    u: f32,
    v: f32,
}

// Constants for D65 white point
const XN: f32 = 95.047;
const YN: f32 = 100.0;
const ZN: f32 = 108.883;

/// Helper function to perform linear interpolation
fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + t * (end - start)
}

/// Convert sRGB to linear RGB
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert linear RGB to sRGB
fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Convert RGB to XYZ
fn rgb_to_xyz(rgb: RGB) -> XYZ {
    let r = srgb_to_linear(rgb.r / 255.0);
    let g = srgb_to_linear(rgb.g / 255.0);
    let b = srgb_to_linear(rgb.b / 255.0);

    XYZ {
        x: r * 41.24 + g * 35.76 + b * 18.05,
        y: r * 21.26 + g * 71.52 + b * 7.22,
        z: r * 1.93 + g * 11.92 + b * 95.05,
    }
}

/// Convert XYZ to RGB
fn xyz_to_rgb(xyz: XYZ) -> RGB {
    let r = 3.2406 * xyz.x - 1.5372 * xyz.y - 0.4986 * xyz.z;
    let g = -0.9689 * xyz.x + 1.8758 * xyz.y + 0.0415 * xyz.z;
    let b = 0.0557 * xyz.x - 0.2040 * xyz.y + 1.0570 * xyz.z;

    RGB {
        r: (linear_to_srgb(r) * 255.0).max(0.0).min(255.0),
        g: (linear_to_srgb(g) * 255.0).max(0.0).min(255.0),
        b: (linear_to_srgb(b) * 255.0).max(0.0).min(255.0),
    }
}

/// Convert XYZ to CIELUV
fn xyz_to_cieluv(xyz: XYZ) -> CIELUV {
    let u_prime = 4.0 * xyz.x / (xyz.x + 15.0 * xyz.y + 3.0 * xyz.z);
    let v_prime = 9.0 * xyz.y / (xyz.x + 15.0 * xyz.y + 3.0 * xyz.z);

    let y_ratio = xyz.y / YN;
    let l = if y_ratio > 0.008856 {
        116.0 * y_ratio.powf(1.0 / 3.0) - 16.0
    } else {
        903.3 * y_ratio
    };

    CIELUV {
        l,
        u: 13.0 * l * (u_prime - 0.19783000664283),
        v: 13.0 * l * (v_prime - 0.46831999493879),
    }
}

/// Convert CIELUV to XYZ
fn cieluv_to_xyz(cieluv: CIELUV) -> XYZ {
    if cieluv.l == 0.0 {
        return XYZ { x: 0.0, y: 0.0, z: 0.0 };
    }

    let u_prime = cieluv.u / (13.0 * cieluv.l) + 0.19783000664283;
    let v_prime = cieluv.v / (13.0 * cieluv.l) + 0.46831999493879;

    let y = if cieluv.l > 8.0 {
        YN * ((cieluv.l + 16.0) / 116.0).powi(3)
    } else {
        YN * cieluv.l / 903.3
    };

    let x = y * 9.0 * u_prime / (4.0 * v_prime);
    let z = y * (12.0 - 3.0 * u_prime - 20.0 * v_prime) / (4.0 * v_prime);

    XYZ { x, y, z }
}

/// Interpolate between two CIELUV colors based on a parameter `t` (0.0 to 1.0).
/// `t = 0.0` returns the start color, `t = 1.0` returns the end color.
pub fn interpolate_cieluv(start: CIELUV, end: CIELUV, t: f32) -> CIELUV {
    CIELUV {
        l: lerp(start.l, end.l, t),
        u: lerp(start.u, end.u, t),
        v: lerp(start.v, end.v, t),
    }
}

/// Create a gradient of `n` colors between two RGB colors in CIELUV space
pub fn cieluv_gradient<const N: usize>(start: RGB, end: RGB) -> [RGB; N] {
    let start_xyz = rgb_to_xyz(start);
    let end_xyz = rgb_to_xyz(end);

    let start_luv = xyz_to_cieluv(start_xyz);
    let end_luv = xyz_to_cieluv(end_xyz);
    let mut result= [RGB::default(); N];

    for i in 0..N {
        let t = i as f32 / (N - 1) as f32;
        let interpolated_luv = CIELUV {
            l: lerp(start_luv.l, end_luv.l, t),
            u: lerp(start_luv.u, end_luv.u, t),
            v: lerp(start_luv.v, end_luv.v, t),
        };
        let interpolated_xyz = cieluv_to_xyz(interpolated_luv);
        result[i] = xyz_to_rgb(interpolated_xyz);
    }

    result
}