//! Plumage: the six bodies a Simurgh can wear, and the language colors on its tail.
//!
//! The caller chooses one of six [`SimurghBody`] values. A body sets the color of the
//! breast, neck, and wings, the crest, the beak, and the golden light of the farr. The
//! tail streamers carry the account's languages: each language's GitHub color becomes
//! a four-step ramp from shadow to highlight, and the streamers are shared out among
//! the languages in proportion to their shares, the largest first. Grays and
//! languages without a color wear the body's own plumage. A young Simurgh shows its
//! language colors pale until [`crate::development::FULL_COLOR_CONTRIBUTIONS`].
//!
//! The streak marks from [`crate::markings`] show here too. The longest streak changes
//! only the language colors, from dull through normal to extra bright and vivid.

use crate::development::{FULL_COLOR_CONTRIBUTIONS, SimurghDevelopment};
use crate::markings::{StreakMarks, calculate_streak_marks};

/// One RGBA color.
pub(crate) type Rgba = [u8; 4];

/// A four-step ramp from shadow to highlight.
pub(crate) type Ramp = [Rgba; 4];

const OUTLINE: Rgba = [24, 20, 37, 255];
/// A language color is dull below this vibrancy and vivid above it.
const DULL_SATURATION: f64 = 0.3;
const DULL_DARKENING: f64 = 0.15;
const VIVID_SATURATION_GAIN: f64 = 0.3;
const VIVID_BRIGHTENING: f64 = 0.04;
/// Colors whose saturation is below this count as gray.
const GRAY_SATURATION: f64 = 0.12;

/// The Simurgh's body color, chosen by the profile owner. Each body redraws the breast,
/// wings, crest, beak, and farr in its own palette; the tail keeps the language colors.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SimurghBody {
    /// Zarrin: golden, with a crimson crest.
    #[default]
    Zarrin,
    /// Firuzeh: turquoise, with a golden crest.
    Firuzeh,
    /// Lajvard: lapis lazuli blue, with a golden crest.
    Lajvard,
    /// Sepid: silver white, with a golden crest.
    Sepid,
    /// Shab: the deep blue of night, with a silver crest.
    Shab,
    /// Arghavan: the purple of the Judas tree, with a rose crest.
    Arghavan,
}

/// The colors a body gives the parts that are not language colored.
pub(crate) struct Palette {
    /// Breast, neck, wing, and default tail, shadow to highlight, plus a fifth sheen.
    pub plumage: [Rgba; 5],
    pub crest: [Rgba; 3],
    pub beak: [Rgba; 3],
    /// The golden light of the farr on wing tips, tail rims, and the halo.
    pub farr: [Rgba; 3],
}

impl SimurghBody {
    pub(crate) fn palette(self) -> Palette {
        const GOLD_FARR: [Rgba; 3] = [
            [247, 118, 34, 255],
            [254, 174, 52, 255],
            [254, 231, 97, 255],
        ];
        const WHITE_FARR: [Rgba; 3] = [
            [192, 203, 220, 255],
            [232, 238, 246, 255],
            [255, 255, 255, 255],
        ];
        const GOLD_CREST: [Rgba; 3] = [
            [215, 118, 67, 255],
            [254, 174, 52, 255],
            [254, 231, 97, 255],
        ];
        const GOLD_BEAK: [Rgba; 3] = [[190, 74, 47, 255], [247, 140, 40, 255], [254, 200, 70, 255]];
        match self {
            Self::Zarrin => Palette {
                plumage: [
                    [98, 50, 48, 255],
                    [190, 100, 46, 255],
                    [247, 163, 40, 255],
                    [254, 214, 84, 255],
                    [255, 242, 170, 255],
                ],
                crest: [[120, 24, 44, 255], [200, 40, 60, 255], [240, 90, 90, 255]],
                beak: [[98, 50, 48, 255], [190, 74, 47, 255], [230, 110, 60, 255]],
                farr: WHITE_FARR,
            },
            Self::Firuzeh => Palette {
                plumage: [
                    [10, 62, 84, 255],
                    [14, 120, 140, 255],
                    [30, 185, 190, 255],
                    [110, 231, 220, 255],
                    [200, 252, 240, 255],
                ],
                crest: GOLD_CREST,
                beak: GOLD_BEAK,
                farr: GOLD_FARR,
            },
            Self::Lajvard => Palette {
                plumage: [
                    [20, 28, 78, 255],
                    [30, 60, 150, 255],
                    [52, 104, 200, 255],
                    [110, 160, 235, 255],
                    [196, 220, 252, 255],
                ],
                crest: GOLD_CREST,
                beak: GOLD_BEAK,
                farr: GOLD_FARR,
            },
            Self::Sepid => Palette {
                plumage: [
                    [90, 105, 136, 255],
                    [139, 155, 180, 255],
                    [192, 203, 220, 255],
                    [232, 238, 246, 255],
                    [255, 255, 255, 255],
                ],
                crest: GOLD_CREST,
                beak: GOLD_BEAK,
                farr: GOLD_FARR,
            },
            Self::Shab => Palette {
                plumage: [
                    [14, 12, 28, 255],
                    [34, 36, 70, 255],
                    [60, 66, 112, 255],
                    [100, 110, 160, 255],
                    [160, 172, 210, 255],
                ],
                crest: [
                    [90, 105, 136, 255],
                    [192, 203, 220, 255],
                    [255, 255, 255, 255],
                ],
                beak: GOLD_BEAK,
                farr: GOLD_FARR,
            },
            Self::Arghavan => Palette {
                plumage: [
                    [58, 20, 84, 255],
                    [110, 40, 140, 255],
                    [165, 70, 185, 255],
                    [215, 120, 220, 255],
                    [245, 190, 245, 255],
                ],
                crest: [
                    [160, 40, 110, 255],
                    [230, 90, 160, 255],
                    [255, 170, 210, 255],
                ],
                beak: GOLD_BEAK,
                farr: GOLD_FARR,
            },
        }
    }
}

/// A Simurgh's colors, resolved from its body, languages, and streaks.
pub(crate) struct Coat {
    pub palette: Palette,
    pub marks: StreakMarks,
    /// One ramp per counted language with its share, largest share first. A language
    /// with no usable color wears the body's plumage.
    inks: Vec<(f64, Ramp)>,
    default_ink: Ramp,
}

pub(crate) fn dress(simurgh_development: &SimurghDevelopment, simurgh_body: SimurghBody) -> Coat {
    let palette = simurgh_body.palette();
    let marks = calculate_streak_marks(simurgh_development);
    let default_ink: Ramp = [
        palette.plumage[0],
        palette.plumage[1],
        palette.plumage[2],
        palette.plumage[3],
    ];
    let paleness =
        (simurgh_development.total_contributions as f64 / FULL_COLOR_CONTRIBUTIONS as f64).min(1.0);
    let mut inks: Vec<(f64, Ramp)> = simurgh_development
        .language_shares
        .iter()
        .map(|(language_name, &share)| {
            let ramp = simurgh_development
                .language_colors
                .get(language_name)
                .copied()
                .flatten()
                .and_then(|language_color| language_ramp(language_color.0, &marks, paleness))
                .unwrap_or(default_ink);
            (share, ramp)
        })
        .collect();
    inks.sort_by(|(left_share, _), (right_share, _)| right_share.total_cmp(left_share));
    Coat {
        palette,
        marks,
        inks,
        default_ink,
    }
}

impl Coat {
    /// Choose a ramp for each of `feather_count` tail streamers. Each language gets
    /// streamers in proportion to its share, the largest first, and the seed shuffles
    /// where they sit in the fan. Without languages every streamer wears the body.
    pub(crate) fn assign_tail_inks(&self, feather_count: usize, identity_seed: u64) -> Vec<Ramp> {
        if self.inks.is_empty() || feather_count == 0 {
            return vec![self.default_ink; feather_count];
        }
        let counted = &self.inks[..self.inks.len().min(feather_count)];
        let total_share: f64 = counted.iter().map(|(share, _)| share).sum();
        let mut counts = vec![1_usize; counted.len()];
        for _ in 0..feather_count - counted.len() {
            let neediest = (0..counted.len())
                .max_by(|&left, &right| {
                    let deficit = |index: usize| {
                        counted[index].0 / total_share * feather_count as f64 - counts[index] as f64
                    };
                    deficit(left)
                        .total_cmp(&deficit(right))
                        .then(right.cmp(&left))
                })
                .expect("counted languages are not empty");
            counts[neediest] += 1;
        }
        let mut sequence: Vec<usize> = counts
            .iter()
            .enumerate()
            .flat_map(|(index, &count)| std::iter::repeat_n(index, count))
            .collect();
        let mut state = identity_seed ^ 0x9E37_79B9_7F4A_7C15;
        for position in (1..sequence.len()).rev() {
            let chosen = (splitmix64(&mut state) % (position as u64 + 1)) as usize;
            sequence.swap(position, chosen);
        }
        sequence
            .into_iter()
            .map(|language_index| counted[language_index].1)
            .collect()
    }

    /// The ramp for plumage, and a seed-keyed ramp for any language, as the seeds
    /// that fall from the tree use.
    pub(crate) fn seed_ink(&self, seed_index: usize) -> Ramp {
        if self.inks.is_empty() {
            self.default_ink
        } else {
            self.inks[seed_index % self.inks.len()].1
        }
    }
}

/// The outline color shared by every shape.
pub(crate) const fn outline() -> Rgba {
    OUTLINE
}

/// A deterministic 64-bit generator step.
pub(crate) fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut mixed = *state;
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    mixed ^ (mixed >> 31)
}

/// A language color as a shadow-to-highlight ramp, or `None` for grays.
fn language_ramp(language_color: [u8; 3], marks: &StreakMarks, paleness: f64) -> Option<Ramp> {
    let (hue, saturation, lightness) = rgb_to_hsl(language_color);
    if saturation < GRAY_SATURATION {
        return None;
    }
    let vibrancy = marks.color_vibrancy;
    let (saturation_scale, lightness_shift) = if vibrancy < 1.0 {
        (
            DULL_SATURATION + (1.0 - DULL_SATURATION) * vibrancy,
            -DULL_DARKENING * (1.0 - vibrancy),
        )
    } else {
        (
            1.0 + VIVID_SATURATION_GAIN * (vibrancy - 1.0),
            VIVID_BRIGHTENING * (vibrancy - 1.0),
        )
    };
    let pale_saturation = 0.4 + 0.6 * paleness;
    let saturation = (saturation * 1.2 * saturation_scale * pale_saturation).clamp(0.0, 1.0);
    let base_lightness = (lightness.clamp(0.48, 0.68) + lightness_shift).clamp(0.3, 0.78);
    let lightness = base_lightness + (0.82 - base_lightness) * (1.0 - paleness) * 0.6;
    let step = |lightness_scale: f64, saturation_scale: f64, lift: f64| {
        let shaded = (lightness * lightness_scale + lift).clamp(0.05, 0.95);
        let [red, green, blue] = hsl_to_rgb(hue, (saturation * saturation_scale).min(1.0), shaded);
        [red, green, blue, 255]
    };
    Some([
        step(0.58, 1.05, 0.0),
        step(0.82, 1.05, 0.0),
        step(1.0, 1.05, 0.0),
        step(1.0, 0.95, 0.12),
    ])
}

fn rgb_to_hsl([red, green, blue]: [u8; 3]) -> (f64, f64, f64) {
    let (red, green, blue) = (
        f64::from(red) / 255.0,
        f64::from(green) / 255.0,
        f64::from(blue) / 255.0,
    );
    let high = red.max(green).max(blue);
    let low = red.min(green).min(blue);
    let lightness = (high + low) / 2.0;
    let spread = high - low;
    if spread == 0.0 {
        return (0.0, 0.0, lightness);
    }
    let saturation = spread / (1.0 - (2.0 * lightness - 1.0).abs());
    let sector = if high == red {
        ((green - blue) / spread).rem_euclid(6.0)
    } else if high == green {
        (blue - red) / spread + 2.0
    } else {
        (red - green) / spread + 4.0
    };
    (sector * 60.0, saturation, lightness)
}

fn hsl_to_rgb(hue: f64, saturation: f64, lightness: f64) -> [u8; 3] {
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let sector = hue.rem_euclid(360.0) / 60.0;
    let second = chroma * (1.0 - (sector.rem_euclid(2.0) - 1.0).abs());
    let (red, green, blue) = match sector as u32 {
        0 => (chroma, second, 0.0),
        1 => (second, chroma, 0.0),
        2 => (0.0, chroma, second),
        3 => (0.0, second, chroma),
        4 => (second, 0.0, chroma),
        _ => (chroma, 0.0, second),
    };
    let offset = lightness - chroma / 2.0;
    [red, green, blue].map(|channel| ((channel + offset) * 255.0).round().clamp(0.0, 255.0) as u8)
}
