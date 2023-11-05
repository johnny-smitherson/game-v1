#![crate_type = "dylib"]

pub const NOISE_SEED: i32 = 11;
pub const MOUNTAIN_HEIGHT: f32 = 500.0;
pub const NOISE_BASE_FREQ: f32 = 100.0;

/// returns single noise value for unscaled position. noise is capped [-1, 1]
fn noise_single(x: f32, y: f32, seed: i32) -> f32 {
    use simdnoise::NoiseBuilder;
    // size = 1 does not work gives NaN
    NoiseBuilder::fbm_2d_offset(x, 2, y, 2)
        .with_seed(seed)
        .generate_scaled(-1.0, 1.0)[0]
}

pub fn height(x: f32, y: f32) -> f32 {
    let octaves = 2;
    let count_per_octave = 2;
    (0..octaves)
        .map(|i| {
            let exp = 1.3_f32.powi(i);
            let freq = NOISE_BASE_FREQ * exp;
            let height = MOUNTAIN_HEIGHT / exp;
            (0..count_per_octave)
                .map(|j| noise_single(x / freq, y / freq, (j + 1) * NOISE_SEED + i) * height)
                .sum::<f32>()
        })
        .sum::<f32>()
        / octaves as f32

}
