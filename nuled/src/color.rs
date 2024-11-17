/// Color manipulation library.
///
/// Allows conversion between RGB, XYZ and CIELUV color spaces,
/// as well as creation of gradients through the CIELUV color space.

use num_traits::Float;

/// Represents a color in the sRGB color space.
///
/// Values in the range of 0.0..255.0.
///
/// * `r` is the amount of red,
/// * `g` is the amount of green,
/// * `b` is the amount of blue.
#[derive(Default, Debug, Clone, Copy)]
pub struct RGB {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl From<XYZ> for RGB {
    fn from(xyz: XYZ) -> Self {
        // sYCC: Amendment 1 to IEC 61966-2-1:1999.
        // Higher conversion precision with seven decimals.
        let r = 3.2406255 * xyz.x - 1.5372080 * xyz.y - 0.4986286 * xyz.z;
        let g = -0.9689307 * xyz.x + 1.8758561 * xyz.y + 0.0415175 * xyz.z;
        let b = 0.0557101 * xyz.x - 0.2040211 * xyz.y + 1.0570959 * xyz.z;

        Self {
            r: (linear_to_srgb(r) * 255.0).clamp(0.0, 255.0),
            g: (linear_to_srgb(g) * 255.0).clamp(0.0, 255.0),
            b: (linear_to_srgb(b) * 255.0).clamp(0.0, 255.0),
        }
    }
}

/// Conversions to and from CIELUV/RGB is done through the XYZ color space.
impl From<CIELUV> for RGB {
    fn from(cieluv: CIELUV) -> Self {
        XYZ::from(cieluv).into()
    }
}

/// Conversions to and from HCL/RGB is done via the CIELUV color space.
impl From<HCL> for RGB {
    fn from(hcl: HCL) -> Self {
        CIELUV::from(hcl).into()
    }
}

impl Into<smart_leds::RGB8> for RGB {
    fn into(self) -> smart_leds::RGB8 {
        smart_leds::RGB8 {
            r: self.r.round() as u8,
            g: self.g.round() as u8,
            b: self.b.round() as u8,
        }
    }
}

/// CIE 1931 XYZ color space, derived from CIE RGB in an effort to simplify the math.
/// This color space defines the relationship between the visible spectrum
/// and the visual sensation of specific colors by human color vision.
///
/// Values in the range of 0.0..1.0.
///
/// * `x` is a mix of all three RGB curves chosen to be nonnegative,
/// * `y` is the luminance, and
/// * `z` is quasi-equal to blue (from CIE RGB).
#[derive(Debug, Default, Clone, Copy)]
pub struct XYZ {
    x: f32,
    y: f32,
    z: f32,
}

// Constants for D65 white point
const X_REF: f32 = 95.047;
const Y_REF: f32 = 100.0;
const Z_REF: f32 = 108.883;

// XYZ/LUV conversion
const K: f32 = 24389.0 / 27.0;
const E: f32 = 216.0 / 24389.0;
const U_PRIME_REF: f32 = 4.0 * X_REF / (X_REF + 15.0 * Y_REF + 3.0 * Z_REF);
const V_PRIME_REF: f32 = 9.0 * Y_REF / (X_REF + 15.0 * Y_REF + 3.0 * Z_REF);

impl From<RGB> for XYZ {
    fn from(rgb: RGB) -> Self {
        let r = srgb_to_linear(rgb.r / 255.0);
        let g = srgb_to_linear(rgb.g / 255.0);
        let b = srgb_to_linear(rgb.b / 255.0);

        // Based on sRGB Working Space Matrix
        // http://www.brucelindbloom.com/Eqn_RGB_XYZ_Matrix.html
        Self {
            x: r * 0.4124564 + g * 0.3575761 + b * 0.1804375,
            y: r * 0.2126729 + g * 0.7151522 + b * 0.0721750,
            z: r * 0.0193339 + g * 0.1191920 + b * 0.9503041,
        }
    }
}

impl From<CIELUV> for XYZ {
    fn from(cieluv: CIELUV) -> Self {
        if cieluv.l == 0.0 {
            return XYZ { x: 0.0, y: 0.0, z: 0.0 };
        }

        let u_prime = cieluv.u / (13.0 * cieluv.l) + 0.19783000664283;
        let v_prime = cieluv.v / (13.0 * cieluv.l) + 0.46831999493879;

        let y = if cieluv.l > 8.0 {
            Y_REF * ((cieluv.l + 16.0) / 116.0).powi(3)
        } else {
            Y_REF * cieluv.l / 903.3
        };

        let x = y * 9.0 * u_prime / (4.0 * v_prime);
        let z = y * (12.0 - 3.0 * u_prime - 20.0 * v_prime) / (4.0 * v_prime);

        XYZ { x, y, z }
    }
}

/// Represents a color using the CIE 1976 L*, u*, v* color space.
///
/// * `l` is the luminance, with values `0.0..1.0`,
/// * `u` is the horizontal axis (green/red), with values approximately `-1.34..2.24`, and
/// * `v` is the vertical axis (blue/yellow), with values approximately `-1.40..1.22`.
///
#[derive(Default, Debug, Clone, Copy)]
pub struct CIELUV {
    l: f32,
    u: f32,
    v: f32,
}

impl CIELUV {
    /// Interpolate between two colors based on a parameter `t` (0.0 to 1.0).
    /// `t = 0.0` returns the start color, `t = 1.0` returns the end color.
    pub fn interpolate(&self, end: &Self, t: f32) -> Self {
        Self {
            l: lerp(self.l, end.l, t),
            u: lerp(self.u, end.u, t),
            v: lerp(self.v, end.v, t),
        }
    }
}

impl From<XYZ> for CIELUV {
    // Verified here: http://www.brucelindbloom.com/index.html?Eqn_XYZ_to_Luv.html
    // Introduced constants due to http://www.brucelindbloom.com/LContinuity.html
    fn from(xyz: XYZ) -> Self {
        if xyz.x == 0.0 && xyz.y == 0.0 && xyz.z == 0.0 {
            return Self { l: 0.0, u: 0.0, v: 0.0 };
        }

        let u_prime = 4.0 * xyz.x / (xyz.x + 15.0 * xyz.y + 3.0 * xyz.z);
        let v_prime = 9.0 * xyz.y / (xyz.x + 15.0 * xyz.y + 3.0 * xyz.z);

        let y_ref = xyz.y / Y_REF;
        let l = if y_ref > E {
            116.0 * y_ref.powf(1.0 / 3.0) - 16.0
        } else {
            K * y_ref
        };

        Self {
            l,
            u: 13.0 * l * (u_prime - U_PRIME_REF),
            v: 13.0 * l * (v_prime - V_PRIME_REF),
        }
    }
}

/// Conversions to and from CIELUV/RGB is done through the XYZ color space.
impl From<RGB> for CIELUV {
    fn from(rgb: RGB) -> Self {
        XYZ::from(rgb).into()
    }
}

impl From<HCL> for CIELUV {
    fn from(hcl: HCL) -> Self {
        let h_rad = hcl.h.to_radians(); // Convert hue to radians
        let u = hcl.c * h_rad.cos();
        let v = hcl.c * h_rad.sin();
        CIELUV { l: hcl.l, u, v }
    }
}

/// Cylindrical representation of the CIELUV color space.
///
/// * `h` is the hue, ranging from `0.0..360.0`,
/// * `c` is the chromaticity, ranging from `0.0..1.0`, and
/// * `l` is the luminance, ranging from `0.0..1.0`.
///
#[derive(Debug, Default, Clone, Copy)]
pub struct HCL {
    pub h: f32,
    pub c: f32,
    pub l: f32,
}

/// Helper function to perform linear interpolation
pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + t * (end - start)
}

/// Convert sRGB to linear RGB (inverse sRGB companding)
/// Verified here: http://www.brucelindbloom.com/index.html?Eqn_RGB_to_XYZ.html
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert linear RGB to sRGB
/// Verified here: http://www.brucelindbloom.com/index.html?Eqn_XYZ_to_RGB.html
fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}