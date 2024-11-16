/// Color manipulation library.
///
/// Allows conversion between RGB, XYZ and CIELUV color spaces,
/// as well as creation of gradients through the CIELUV color space.

use num_traits::Float;

/// Red, green, blue.
#[derive(Default, Debug, Clone, Copy)]
pub struct RGB {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl From<XYZ> for RGB {
    fn from(xyz: XYZ) -> Self {
        // not verified...
        let r = 3.2406 * xyz.x - 1.5372 * xyz.y - 0.4986 * xyz.z;
        let g = -0.9689 * xyz.x + 1.8758 * xyz.y + 0.0415 * xyz.z;
        let b = 0.0557 * xyz.x - 0.2040 * xyz.y + 1.0570 * xyz.z;

        Self {
            r: (linear_to_srgb(r) * 255.0).max(0.0).min(255.0),
            g: (linear_to_srgb(g) * 255.0).max(0.0).min(255.0),
            b: (linear_to_srgb(b) * 255.0).max(0.0).min(255.0),
        }
    }
}

impl From<CIELUV> for RGB {
    fn from(cieluv: CIELUV) -> Self {
        XYZ::from(cieluv).into()
    }
}

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

/// Hue, saturation, value.
#[derive(Default, Debug, Clone, Copy)]
pub struct HSV {
    pub hue: u8,
    pub sat: u8,
    pub val: u8,
}

/// From the smart_leds crate.
///
/// Converts a hsv value into RGB values. Because the hsv values are integers, the precision of the
/// resulting RGB value is limited to ±4.
///
/// NOTE: Since most led protocols & their implementations are very timing
/// sensitive, it's advisable to do the conversion before `write`-ing.
///
/// # Example
/// ```
/// use led::{hsv2rgb, Hsv};
/// let hsv = Hsv{hue: 89, sat: 230, val: 42};
/// let conv_rgb = hsv2rgb(hsv);
/// // will return RGB { r: 4, g: 41, b: 8},
/// ```
impl Into<RGB> for HSV {
    fn into(self) -> RGB {
        let v: u16 = self.val as u16;
        let s: u16 = self.sat as u16;
        let f: u16 = (self.hue as u16 * 2 % 85) * 3; // relative interval

        let p: u16 = v * (255 - s) / 255;
        let q: u16 = v * (255 - (s * f) / 255) / 255;
        let t: u16 = v * (255 - (s * (255 - f)) / 255) / 255;
        match self.hue {
            0..=42 => RGB {
                r: v as f32,
                g: t as f32,
                b: p as f32,
            },
            43..=84 => RGB {
                r: q as f32,
                g: v as f32,
                b: p as f32,
            },
            85..=127 => RGB {
                r: p as f32,
                g: v as f32,
                b: t as f32,
            },
            128..=169 => RGB {
                r: p as f32,
                g: q as f32,
                b: v as f32,
            },
            170..=212 => RGB {
                r: t as f32,
                g: p as f32,
                b: v as f32,
            },
            213..=254 => RGB {
                r: v as f32,
                g: p as f32,
                b: q as f32,
            },
            255 => RGB {
                r: v as f32,
                g: t as f32,
                b: p as f32,
            },
        }
    }
}

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

#[derive(Default, Debug, Clone, Copy)]
pub struct CIELUV {
    l: f32,
    u: f32,
    v: f32,
}

impl CIELUV {
    /// Interpolate between two CIELUV colors based on a parameter `t` (0.0 to 1.0).
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

/// Hue, chroma, luminance.
/// https://cscheid.github.io/lux/demos/hcl/hcl.html
#[derive(Debug, Default, Clone, Copy)]
pub struct HCL {
    /// Hue in degrees, 0.0..360.0.
    pub h: f32,
    pub c: f32,
    pub l: f32,
}

impl HCL {
    /// Convert HCL directly to XYZ without using the CIELUV space.
    /// Minimal intermediary calculations reduce computational overhead.
    ///
    /// FIXME: this is not working as it should
    ///
    /// ## Simplifications:
    ///
    /// This algorithm skips a full CIELUV calculation and uses direct approximations for chromatic components.
    /// Lightness (L∗) is mapped directly to Y, while chromatic components (u′, v′) influence X and Z.
    pub fn to_xyz_fast(&self) -> XYZ {
        // Convert H to radians
        let h_rad = self.h.to_radians();

        // Compute chromatic components u' and v'
        let u = self.c * h_rad.cos();
        let v = self.c * h_rad.sin();

        // Convert to XYZ space
        let y = if self.l > 7.999_592 { ((self.l + 16.0) / 116.0).powi(3) } else { self.l / 903.3 };
        let x = y + u / 13.0 / (self.l / 116.0);
        let z = y - v / 13.0 / (self.l / 116.0);

        XYZ { x, y, z }
    }

    pub fn to_rgb_fast(&self) -> RGB {
        self.to_xyz_fast().into()
    }
}

/// Helper function to perform linear interpolation
fn lerp(start: f32, end: f32, t: f32) -> f32 {
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