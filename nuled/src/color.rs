/// The following code was first written by GPT-o4,
/// and subsequently modified.

use num_traits::Float;
use heapless::String;
use core::fmt::Write;

/// Red, green, blue.
#[derive(Default, Debug, Clone, Copy)]
pub struct RGB {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl RGB {
    /// Produce a comma-separated value, suitable for OpenHAB.
    pub fn serialize(&self) -> Option<String<11>> {
        let mut s = String::new();
        write!(s, "{},{},{}", self.r, self.g, self.b).ok()?;
        Some(s)
    }

    /// Parse a comma-separated value from OpenHAB.
    pub fn parse(data: &[u8]) -> Option<Self> {
        let mut iter = data.iter();
        let r = Self::parse_int_and_delimiter(&mut iter)? as f32;
        let g = Self::parse_int_and_delimiter(&mut iter)? as f32;
        let b = Self::parse_int_and_delimiter(&mut iter)? as f32;
        Some(Self { r, g, b })
    }

    /// Parse a single number up to three digits wide,
    /// and optionally a comma separator, unless end of line is reached.
    fn parse_int_and_delimiter<'a>(mut iter: impl Iterator<Item=&'a u8>) -> Option<u8> {
        use core::str::FromStr;
        use heapless::String;
        let mut number_string = String::<3>::new();

        loop {
            let char = match iter.next() {
                None => None,
                Some(c) if *c as char == ',' => None,
                Some(c) => Some(*c as char),
            };

            match char {
                None => {
                    break;
                }
                Some(char) => {
                    if let Err(_) = number_string.push(char) {
                        return None;
                    };
                }
            }
        }

        u8::from_str(number_string.as_str()).ok()
    }
}

impl From<XYZ> for RGB {
    fn from(xyz: XYZ) -> Self {
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
#[derive(Copy, Clone, Default)]
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

#[derive(Debug, Clone, Copy)]
pub struct XYZ {
    x: f32,
    y: f32,
    z: f32,
}

impl From<RGB> for XYZ {
    fn from(rgb: RGB) -> Self {
        let r = srgb_to_linear(rgb.r / 255.0);
        let g = srgb_to_linear(rgb.g / 255.0);
        let b = srgb_to_linear(rgb.b / 255.0);

        Self {
            x: r * 41.24 + g * 35.76 + b * 18.05,
            y: r * 21.26 + g * 71.52 + b * 7.22,
            z: r * 1.93 + g * 11.92 + b * 95.05,
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
            YN * ((cieluv.l + 16.0) / 116.0).powi(3)
        } else {
            YN * cieluv.l / 903.3
        };

        let x = y * 9.0 * u_prime / (4.0 * v_prime);
        let z = y * (12.0 - 3.0 * u_prime - 20.0 * v_prime) / (4.0 * v_prime);

        XYZ { x, y, z }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CIELUV {
    l: f32,
    u: f32,
    v: f32,
}

impl CIELUV {
    /// Interpolate between two CIELUV colors based on a parameter `t` (0.0 to 1.0).
    /// `t = 0.0` returns the start color, `t = 1.0` returns the end color.
    pub fn interpolate(&self, end: Self, t: f32) -> Self {
        Self {
            l: lerp(self.l, end.l, t),
            u: lerp(self.u, end.u, t),
            v: lerp(self.v, end.v, t),
        }
    }
}

impl From<XYZ> for CIELUV {
    fn from(xyz: XYZ) -> Self {
        let u_prime = 4.0 * xyz.x / (xyz.x + 15.0 * xyz.y + 3.0 * xyz.z);
        let v_prime = 9.0 * xyz.y / (xyz.x + 15.0 * xyz.y + 3.0 * xyz.z);

        let y_ratio = xyz.y / YN;
        let l = if y_ratio > 0.008856 {
            116.0 * y_ratio.powf(1.0 / 3.0) - 16.0
        } else {
            903.3 * y_ratio
        };

        Self {
            l,
            u: 13.0 * l * (u_prime - 0.19783000664283),
            v: 13.0 * l * (v_prime - 0.46831999493879),
        }
    }
}

impl From<RGB> for CIELUV {
    fn from(rgb: RGB) -> Self {
        XYZ::from(rgb).into()
    }
}

// Constants for D65 white point
//const XN: f32 = 95.047;
const YN: f32 = 100.0;
//const ZN: f32 = 108.883;

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