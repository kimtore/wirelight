use heapless::String;
use core::fmt::Write;

struct Strip<const N: usize>([RGB; N]);

#[derive(Default)]
struct Rainbow<const N: usize> {
    seq_no: usize,
}

impl<const N: usize> Default for Strip<N> {
    fn default() -> Self {
        Self::fill(RGB::default())
    }
}

impl<const N: usize> Strip<N> {
    fn fill(pixel: impl Into<RGB>) -> Self {
        Self([pixel.into(); N])
    }
}

impl<const N: usize> Iterator for Rainbow<N> {
    type Item = Strip<N>;

    fn next(&mut self) -> Option<Self::Item> {
        self.seq_no += 1;
        Some(Strip::fill(HSV {
            hue: (self.seq_no % N) as u8,
            sat: 0,
            val: 0,
        }))
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
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
        let r = Self::parse_int_and_delimiter(&mut iter)?;
        let g = Self::parse_int_and_delimiter(&mut iter)?;
        let b = Self::parse_int_and_delimiter(&mut iter)?;
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
                r: v as u8,
                g: t as u8,
                b: p as u8,
            },
            43..=84 => RGB {
                r: q as u8,
                g: v as u8,
                b: p as u8,
            },
            85..=127 => RGB {
                r: p as u8,
                g: v as u8,
                b: t as u8,
            },
            128..=169 => RGB {
                r: p as u8,
                g: q as u8,
                b: v as u8,
            },
            170..=212 => RGB {
                r: t as u8,
                g: p as u8,
                b: v as u8,
            },
            213..=254 => RGB {
                r: v as u8,
                g: p as u8,
                b: q as u8,
            },
            255 => RGB {
                r: v as u8,
                g: t as u8,
                b: p as u8,
            },
        }
    }
}