use crate::gfx::Color;

use super::block_tables::{OCTANTS, SEXTANTS};

/// Which Unicode block characters are used to draw pixels into a terminal cell
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlockMode {
    /// 2x2 sub-cells, works in every terminal
    #[default]
    Quadrant,
    /// 2x3 sub-cells, needs a terminal or font with Unicode 13 sextants
    Sextant,
    /// 2x4 sub-cells, needs a terminal or font with Unicode 16 octants
    Octant,
}

impl BlockMode {
    pub fn parse(value: &str) -> Option<BlockMode> {
        match value {
            "quadrant" => Some(BlockMode::Quadrant),
            "sextant" => Some(BlockMode::Sextant),
            "octant" => Some(BlockMode::Octant),
            _ => None,
        }
    }
}

/// Pixels of a cell in row order: 2 columns by 4 rows
pub type CellPixels = [Color; 8];

fn channels(color: &Color) -> [u8; 3] {
    [color.r, color.g, color.b]
}

/// Find the value which best separates samples into two groups (Otsu's method).
///
/// A plain midpoint flips between neighboring cells on smooth gradients when
/// a sample sits right on it, which shows up as speckles.
fn threshold(values: impl Iterator<Item = u8>) -> u8 {
    let mut sorted = [0u8; 8];
    let mut count = 0;

    for value in values {
        sorted[count] = value;
        count += 1;
    }

    let sorted = &mut sorted[..count];

    sorted.sort_unstable();

    let total: u32 = sorted.iter().map(|&value| value as u32).sum();
    let mut low_sum = 0u32;
    let mut best = (0.0f32, sorted[count - 1]);

    for split in 1..count {
        low_sum += sorted[split - 1] as u32;

        if sorted[split] == sorted[split - 1] {
            continue;
        }

        let (low, high) = (split as f32, (count - split) as f32);
        let delta = low_sum as f32 / low - (total - low_sum) as f32 / high;
        let score = low * high * delta * delta;

        if score > best.0 {
            best = (score, sorted[split - 1]);
        }
    }

    best.1
}

/// Split pixels into two colors, and a mask of the ones using the foreground.
///
/// Splitting on the channel with the widest range rather than on luminance
/// keeps edges between colors of similar brightness, like red on green.
fn split(pixels: &[Color]) -> (usize, Color, Color) {
    let (mut min, mut max) = ([u8::MAX; 3], [u8::MIN; 3]);

    for pixel in pixels {
        for (channel, value) in channels(pixel).into_iter().enumerate() {
            min[channel] = min[channel].min(value);
            max[channel] = max[channel].max(value);
        }
    }

    let channel = (0..3).max_by_key(|&c| max[c] - min[c]).unwrap_or(0);
    let threshold = threshold(pixels.iter().map(|pixel| channels(pixel)[channel]));
    let mut mask = 0;
    let mut sums = [[0u32; 4]; 2];

    for (index, pixel) in pixels.iter().enumerate() {
        let [r, g, b] = channels(pixel);
        let set = channels(pixel)[channel] > threshold;

        if set {
            mask |= 1 << index;
        }

        let sum = &mut sums[set as usize];

        sum[0] += r as u32;
        sum[1] += g as u32;
        sum[2] += b as u32;
        sum[3] += 1;
    }

    let average = |[r, g, b, count]: [u32; 4]| {
        Color::new(
            ((r + count / 2) / count) as u8,
            ((g + count / 2) / count) as u8,
            ((b + count / 2) / count) as u8,
        )
    };
    let background = average(sums[0]);
    let foreground = if sums[1][3] > 0 {
        average(sums[1])
    } else {
        background
    };

    (mask, background, foreground)
}

fn mix(a: Color, b: Color, weight_a: u16, weight_b: u16) -> Color {
    let total = weight_a + weight_b;
    let channel = |a: u8, b: u8| {
        ((a as u16 * weight_a + b as u16 * weight_b + total / 2) / total) as u8
    };

    Color::new(channel(a.r, b.r), channel(a.g, b.g), channel(a.b, b.b))
}

/// Turn a cell's pixels into two colors and a sextant, resampling 4 rows to 3
pub fn binarize_sextant(p: &CellPixels) -> (&'static str, Color, Color) {
    let rows = [
        mix(p[0], p[2], 3, 1),
        mix(p[1], p[3], 3, 1),
        mix(p[2], p[4], 1, 1),
        mix(p[3], p[5], 1, 1),
        mix(p[6], p[4], 3, 1),
        mix(p[7], p[5], 3, 1),
    ];
    let (mask, background, foreground) = split(&rows);

    (SEXTANTS[mask], background, foreground)
}

/// Turn a cell's pixels into two colors and an octant
pub fn binarize_octant(pixels: &CellPixels) -> (&'static str, Color, Color) {
    let (mask, background, foreground) = split(pixels);

    (OCTANTS[mask], background, foreground)
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: Color = Color::new(255, 255, 255);
    const K: Color = Color::new(0, 0, 0);

    #[test]
    fn uniform_cell_is_a_space() {
        assert_eq!(binarize_sextant(&[W; 8]), (" ", W, W));
        assert_eq!(binarize_octant(&[K; 8]), (" ", K, K));
    }

    #[test]
    fn half_blocks_use_their_existing_characters() {
        let left = [W, K, W, K, W, K, W, K];
        let right = [K, W, K, W, K, W, K, W];

        assert_eq!(binarize_sextant(&left).0, "▌");
        assert_eq!(binarize_octant(&left).0, "▌");
        assert_eq!(binarize_sextant(&right).0, "▐");
        assert_eq!(binarize_octant(&right).0, "▐");
    }

    #[test]
    fn octant_keeps_a_single_row() {
        // Only the top row is lit, which a quadrant cannot show
        let top = [W, W, K, K, K, K, K, K];

        assert_eq!(binarize_octant(&top), ("\u{1FB82}", K, W));
    }

    #[test]
    fn sextant_shows_two_of_three_rows() {
        let top = [W, W, W, W, W, W, K, K];

        assert_eq!(binarize_sextant(&top).0, "\u{1FB0E}");
    }

    #[test]
    fn splits_colors_of_equal_luminance() {
        let red = Color::new(200, 0, 0);
        let blue = Color::new(0, 0, 200);
        let (glyph, background, foreground) =
            binarize_octant(&[red, blue, red, blue, red, blue, red, blue]);

        // Both colors survive, averaging them would give a flat purple
        assert_eq!((glyph, background, foreground), ("▐", red, blue));
    }

    #[test]
    fn smooth_gradient_splits_whole_rows() {
        // A vertical gradient that drifts slightly from left to right
        let gray = |value| Color::new(value, value, value);
        let pixels = [
            gray(14),
            gray(10),
            gray(14),
            gray(10),
            gray(54),
            gray(50),
            gray(94),
            gray(90),
        ];

        // The middle row must not get split in two
        let (mask, _, _) = split(&[
            gray(14),
            gray(10),
            gray(54),
            gray(50),
            gray(94),
            gray(90),
        ]);

        assert!(matches!(mask, 0b110000 | 0b111100));
        assert_ne!(binarize_sextant(&pixels).1, binarize_sextant(&pixels).2);
    }

    #[test]
    fn tables_have_no_duplicates() {
        let unique = |table: &[&str]| {
            let mut sorted = table.to_vec();

            sorted.sort();
            sorted.dedup();
            sorted.len()
        };

        assert_eq!(unique(&SEXTANTS), 64);
        assert_eq!(unique(&OCTANTS), 256);
    }
}
