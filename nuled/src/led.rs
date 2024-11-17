use core::convert::Into;
use num_traits::float::Float;
use crate::color::{lerp, CIELUV, HCL, RGB};

/// One degree through a full circle, expressed in radians.
const ONE_DEGREE_RAD: f32 = core::f32::consts::TAU / 360.0;

/// Global LED params.
///
/// To make the OpenHAB API extremely simple, we define a set of common parameters.
/// Effects may use as many as these as they need.
#[derive(Copy, Clone, Debug)]
pub struct LedEffectParams {
    pub color1: RGB,
    pub color2: RGB,
    pub chroma: f32,
    pub luminance: f32,
    pub size: f32,
    pub speed: f32,
}

impl Default for LedEffectParams {
    fn default() -> Self {
        Self {
            color1: RGB::default(),
            color2: RGB::default(),
            chroma: 0.6,
            luminance: 0.6,
            size: 0.5,
            speed: 0.5,
        }
    }
}

pub trait LedEffect<const N: usize>: Iterator<Item=Strip<N>> {
    fn configure(&mut self, params: LedEffectParams);
}

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

/// Circle through the HCL color space for rainbow colors.
#[derive(Default)]
pub struct Rainbow<const N: usize> {
    chroma: f32,
    luminance: f32,
    degrees: f32,
    degree_velocity: f32,
    /// Degree separation between LEDs to have entire spectrum across strip
    separation: f32,
}

impl<const N: usize> LedEffect<N> for Rainbow<N> {
    fn configure(&mut self, params: LedEffectParams) {
        self.chroma = params.chroma;
        self.luminance = params.luminance;
        self.degree_velocity = lerp(0.0, 3.0, params.speed);
        self.separation = lerp(0.0, 360.0 / N as f32, 1.0 - params.size);
    }
}

impl<const N: usize> Iterator for Rainbow<N> {
    type Item = Strip<N>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut strip = Strip::<N>::default();
        for i in 0..N {
            strip.0[i] = HCL {
                h: self.degrees + self.separation * i as f32,
                c: self.chroma,
                l: self.luminance,
            }.into();
        }
        self.degrees += self.degree_velocity;
        Some(strip)
    }
}

/// Solid color.
#[derive(Default)]
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

impl<const N: usize> LedEffect<N> for Solid<N> {
    fn configure(&mut self, params: LedEffectParams) {
        self.color = params.color1;
        self.finished = false;
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
    start_color: CIELUV,
    end_color: CIELUV,
}

impl<const N: usize> Default for Polyrhythm<N> {
    fn default() -> Self {
        Self {
            spinners: [Spinner::default(); N],
            start_color: CIELUV::default(),
            end_color: CIELUV::default(),
        }
    }
}

impl<const N: usize> LedEffect<N> for Polyrhythm<N> {
    fn configure(&mut self, params: LedEffectParams) {
        self.start_color = params.color1.into();
        self.end_color = params.color2.into();
        for i in 0..N {
            let max_velocity = (ONE_DEGREE_RAD / 5.0) * ((i + 1) as f32);
            self.spinners[i].angular_velocity = lerp(0.0, max_velocity, params.speed);
        }
    }
}

impl<const N: usize> Iterator for Polyrhythm<N> {
    type Item = Strip<N>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut strip = Strip::<N>::default();
        for i in 0..N {
            // translate the range from -1.0..1.0 to 0.0..1.0.
            let amplitude = (1.0 + self.spinners[i].amplitude()) / 2.0;
            let interpolated = self.start_color.interpolate(&self.end_color, amplitude);
            if i == 0 {
                debug!("Polyrhythm: {amplitude:.5} -> {:?}", interpolated);
            }
            strip.0[i] = interpolated.into();
            self.spinners[i].increment();
        }

        Some(strip)
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