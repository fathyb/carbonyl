use crate::gfx::Color;

#[derive(Clone, Copy)]
enum Channel {
    R,
    G,
    B,
}

const COLOR_BUCKETS: usize = 8;
const COLORS: usize = 2_usize.pow(COLOR_BUCKETS as u32);

/// Find the closest color to `color` on `palette` using a binary search
pub fn palette_color(palette: &[Color; COLORS], color: Color) {
    let mut size = palette.len() / 2;
    let mut iter = palette.iter();
    let mut prev = iter.next();
}

fn distance(a: Color, b: Color) {}

pub fn quantize(pixels: &[u8]) -> [Color; COLORS] {
    let mut min = Color::black();
    let mut max = Color::black();
    let mut bucket = Vec::<Color>::new();
    let mut pixels_iter = pixels.iter();
    let mut bucket_iter = bucket.iter_mut();

    // Step 1: find the dominant channel
    loop {
        match (
            bucket_iter.next(),
            pixels_iter.next(),
            pixels_iter.next(),
            pixels_iter.next(),
            pixels_iter.next(),
        ) {
            (Some(color), Some(r), Some(g), Some(b), Some(_)) => {
                // Save the color in a bucket
                color.set_r(*r);
                color.set_g(*g);
                color.set_b(*b);

                min.set_r(min.r().min(color.r()));
                min.set_g(min.g().min(color.g()));
                min.set_b(min.b().min(color.b()));

                max.set_r(max.r().max(color.r()));
                max.set_g(max.g().max(color.g()));
                max.set_b(max.b().max(color.b()));
            }
            _ => break,
        }
    }

    let ranges = [
        (Channel::R, max.r() - min.r()),
        (Channel::G, max.g() - min.g()),
        (Channel::B, max.b() - min.b()),
    ];
    let (channel, _) = ranges
        .iter()
        .reduce(|a, b| if a.1 > b.1 { a } else { b })
        .unwrap();

    // Step 2: perform median-cut
    for i in 1..=COLOR_BUCKETS {
        let buckets = 2_usize.pow(i as u32);
        let size = bucket.len() / buckets;

        for j in 0..buckets {
            let start = j * size;
            let end = start + size;
            let slice = &mut bucket[start..end];

            slice.sort_unstable_by(match channel {
                Channel::R => |a: &Color, b: &Color| a.r().cmp(&b.r()),
                Channel::G => |a: &Color, b: &Color| a.g().cmp(&b.g()),
                Channel::B => |a: &Color, b: &Color| a.b().cmp(&b.b()),
            });
        }
    }

    // Step 3: get the average color in each bucket
    let mut palette = [Color::black(); COLORS];
    let size = bucket.len() / palette.len();

    for (i, color) in palette.iter_mut().enumerate() {
        let start = i * size;
        let end = start + size;
        let slice = &bucket[start..end];
        let mut sum = None;

        for color in slice.into_iter() {
            sum = Some(match sum {
                None => color.cast(),
                Some(sum) => color.cast() + sum,
            })
        }

        if let Some(sum) = sum {
            let avg = sum / size as u32;

            color.set_r(avg.r() as u8);
            color.set_g(avg.g() as u8);
            color.set_b(avg.b() as u8);
        }
    }

    palette
}
