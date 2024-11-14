use num_traits::float::Float;
use crate::color::{HSV, RGB};

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