#[derive(Default, Copy, Clone)]
struct Pixel;

struct Strip<const N: usize>([Pixel; N]);

struct Effect<const N: usize> {}

impl<const N: usize> IntoIterator for Effect<N> {
    type Item = Strip<N>;
    type IntoIter = EffectIterator<N>;

    fn into_iter(self) -> Self::IntoIter {
        EffectIterator::default()
    }
}

#[derive(Default)]
struct EffectIterator<const N: usize> {
    seq_no: u32,
}

impl<const N: usize> Default for Strip<N> {
    fn default() -> Self {
        Self::fill(Pixel)
    }
}

impl<const N: usize> Strip<N> {
    fn fill(pixel: Pixel) -> Self{
        Self([pixel; N])
    }
}

impl<const N: usize> Iterator for EffectIterator<N> {
    type Item = Strip<N>;

    fn next(&mut self) -> Option<Self::Item> {
        self.seq_no += 1;
        let mut strip = Strip::default();
        strip.0[0] = Pixel;
        Some(strip)
    }
}