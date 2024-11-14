use heapless::String;
use core::fmt::Write;
use num_traits::float::Float;

pub struct Strip<const N: usize>(pub [RGB; N]);

impl<const N: usize> Strip<N> {
    pub fn to_rgb8(self) -> [smart_leds::RGB8; N] {
        self.0.map(|x| x.into())
    }
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

/// Loop through all hues on maximum saturation and brightness.
#[derive(Default)]
pub struct Rainbow<const N: usize> {
    seq_no: u8,
}

impl<const N: usize> Iterator for Rainbow<N> {
    type Item = Strip<N>;

    fn next(&mut self) -> Option<Self::Item> {
        self.seq_no += 1;
        Some(Strip::fill(HSV {
            hue: self.seq_no,
            sat: 255,
            val: 255,
        }))
    }
}

/// Solid color.
pub struct Solid<const N: usize> {
    color: RGB,
    finished: bool,
}

impl<const N: usize> Solid<N> {
    pub fn new(color: RGB) -> Self {
        Self {
            color,
            finished: false,
        }
    }
}

impl<const N: usize> Iterator for Solid<N> {
    type Item = Strip<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        self.finished = true;
        Some(Strip::fill(self.color))
    }
}

/// Fade LEDs in and out with a sine wave function.
/// Each LED has a progressively smaller period size.
pub struct Polyrhythm<const N: usize> {
    spinners: [Spinner; N],
}

impl<const N: usize> Polyrhythm<N> {
    pub fn new() -> Self {
        const FRAC: f32 = core::f32::consts::TAU / (360.0 * 6.0);
        let mut spinners = [Spinner::default(); N];
        for i in 0..N {
            spinners[i].angle = FRAC * i as f32;
            spinners[i].angular_velocity = FRAC * ((i + 1) as f32);
        }
        Self {
            spinners,
        }
    }
}

impl<const N: usize> Iterator for Polyrhythm<N> {
    type Item = Strip<N>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut strip = Strip::<N>::default();
        for i in 0..N {
            self.spinners[i].increment();
            // translate the range from -1.0..1.0 to 0.0..1.0.
            let amplitude = (1.0 + self.spinners[i].amplitude()) / 2.0;
            strip.0[i] = HSV {
                hue: 200,
                sat: 255,
                val: float_to_u8(amplitude),
            }.into();
        }

        Some(strip)
    }
}

fn float_to_u8(f: f32) -> u8 {
    (f * 255.0) as u8
}


/// Red, green, blue.
#[derive(Default, Debug, Clone, Copy)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Into<smart_leds::RGB8> for RGB {
    fn into(self) -> smart_leds::RGB8 {
        smart_leds::RGB8 {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
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

#[derive(Debug, Clone, Copy, Default)]
struct Spinner {
    angular_velocity: f32,
    angle: f32,
}

impl Spinner {
    pub fn increment(&mut self) {
        self.angle += self.angular_velocity;
    }

    pub fn amplitude(&self) -> f32 {
        self.angle.sin()
    }
}