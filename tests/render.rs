use std::collections::BTreeSet;
use std::io::Cursor;

use chrono::{Datelike, Days, NaiveDate};
use simurgh::behavior::Behavior;
use simurgh::coat::SimurghBody;
use simurgh::development::{
    MAX_GROWTH_CONTRIBUTIONS, RecentStreak, SimurghDevelopment, derive_simurgh_development,
};
use simurgh::history::{ContributionWeek, LanguageColor};
use simurgh::motion::frame_count;
use simurgh::output::encode_png;
use simurgh::render::{
    FRAME_HEIGHT_PIXELS, FRAME_WIDTH_PIXELS, SimurghFrame, render_flight_frame, render_simurgh,
};

type LanguageMix<'a> = &'a [(&'a str, Option<&'a str>, f64)];

const BODIES: [SimurghBody; 6] = [
    SimurghBody::Zarrin,
    SimurghBody::Firuzeh,
    SimurghBody::Lajvard,
    SimurghBody::Sepid,
    SimurghBody::Shab,
    SimurghBody::Arghavan,
];
const BEHAVIORS: [Behavior; 5] = [
    Behavior::Glide,
    Behavior::Soar,
    Behavior::Perch,
    Behavior::SeedFall,
    Behavior::Sleep,
];
const FOUR_LANGUAGE_MIX: LanguageMix = &[
    ("C", Some("#555555"), 0.25),
    ("Python", Some("#3572a5"), 0.25),
    ("C++", Some("#f34b7d"), 0.25),
    ("JavaScript", Some("#f1e05a"), 0.25),
];
const BLUE_MIX: LanguageMix = &[("Python", Some("#3572a5"), 1.0)];
const GROWTH_STEPS: [u64; 6] = [0, 500, 1_500, 12_750, 50_000, MAX_GROWTH_CONTRIBUTIONS];

#[test]
fn rendering_is_reproducible_and_leaves_the_development_alone() {
    let simurgh_development = develop_with_languages(12_750, FOUR_LANGUAGE_MIX);
    let first = render_simurgh(&simurgh_development, SimurghBody::default());
    assert_eq!(
        first,
        render_simurgh(&simurgh_development, SimurghBody::default())
    );
    assert_eq!(
        render_flight_frame(
            &simurgh_development,
            SimurghBody::Lajvard,
            Behavior::Soar,
            17
        ),
        render_flight_frame(
            &simurgh_development,
            SimurghBody::Lajvard,
            Behavior::Soar,
            17
        )
    );
    assert_eq!(
        simurgh_development,
        develop_with_languages(12_750, FOUR_LANGUAGE_MIX)
    );
}

#[test]
fn languages_color_the_tail_without_changing_the_shape() {
    let plain = render_simurgh(&develop_with_languages(12_750, &[]), SimurghBody::default());
    let mut distinct_images = BTreeSet::new();
    for language_mix in [
        BLUE_MIX,
        FOUR_LANGUAGE_MIX,
        &[("Rust", Some("#dea584"), 1.0)],
        &[
            ("TypeScript", Some("#3178c6"), 0.45),
            ("C++", Some("#f34b7d"), 0.35),
            ("Shell", Some("#89e051"), 0.2),
        ],
    ] {
        let colored = render_simurgh(
            &develop_with_languages(12_750, language_mix),
            SimurghBody::default(),
        );
        assert_ne!(colored, plain);
        assert_eq!(collect_alphas(&colored), collect_alphas(&plain));
        assert!(distinct_images.insert(colored.as_rgba_bytes().to_vec()));
    }
}

#[test]
fn the_largest_language_gets_the_most_tail_streamers() {
    let mix: LanguageMix = &[
        ("C++", Some("#f34b7d"), 0.75),
        ("Python", Some("#3572a5"), 0.25),
    ];
    let simurgh_frame = render_simurgh(&develop_with_languages(12_750, mix), SimurghBody::Sepid);
    let count_where = |matches: fn(&[u8; 4]) -> bool| {
        simurgh_frame
            .as_rgba_bytes()
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel| pixel[3] != 0 && matches(pixel))
            .count()
    };
    let reddish = count_where(|pixel| i32::from(pixel[0]) - i32::from(pixel[2]) > 70);
    let bluish = count_where(|pixel| i32::from(pixel[2]) - i32::from(pixel[0]) > 70);
    assert!(reddish > bluish, "{reddish} against {bluish}");
    assert!(bluish > 0);
}

#[test]
fn a_young_simurgh_shows_pale_language_colors_until_full_color() {
    let blueness = |contributions: u64| {
        render_simurgh(
            &develop_with_languages(contributions, BLUE_MIX),
            SimurghBody::Zarrin,
        )
        .as_rgba_bytes()
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|pixel| pixel[3] != 0)
        .map(|pixel| i32::from(pixel[2]) - (i32::from(pixel[0]) + i32::from(pixel[1])) / 2)
        .max()
        .unwrap()
    };
    assert!(
        blueness(0) + 15 < blueness(500),
        "{} {}",
        blueness(0),
        blueness(500)
    );
}

#[test]
fn bodies_change_colors_but_not_shape() {
    let simurgh_development = develop_with_languages(12_750, FOUR_LANGUAGE_MIX);
    let reference = render_simurgh(&simurgh_development, SimurghBody::default());
    assert_eq!(
        reference,
        render_simurgh(&simurgh_development, SimurghBody::Zarrin)
    );
    let mut distinct_images = BTreeSet::new();
    for simurgh_body in BODIES {
        let simurgh_frame = render_simurgh(&simurgh_development, simurgh_body);
        assert_eq!(collect_alphas(&simurgh_frame), collect_alphas(&reference));
        assert!(distinct_images.insert(simurgh_frame.as_rgba_bytes().to_vec()));
    }
}

#[test]
fn the_current_streak_lights_the_farr_on_every_body() {
    for simurgh_body in BODIES {
        let mut dark = develop_with_languages(12_750, FOUR_LANGUAGE_MIX);
        dark.streaks.recent = None;
        let mut lit = develop_with_languages(12_750, FOUR_LANGUAGE_MIX);
        lit.streaks.recent = Some(RecentStreak::Current(build_weeks(13)));
        let dark_frame = render_simurgh(&dark, simurgh_body);
        let lit_frame = render_simurgh(&lit, simurgh_body);
        assert_ne!(dark_frame, lit_frame, "{simurgh_body:?}");
        // The halo of light adds pixels around the head.
        assert!(
            collect_opaque_pixel_count(&lit_frame) > collect_opaque_pixel_count(&dark_frame),
            "{simurgh_body:?}"
        );
    }
}

#[test]
fn the_farr_grows_with_the_streak_and_fades_after_it_ends() {
    let with_streak = |week_count: usize, ended_days_ago: Option<u64>| {
        let mut simurgh_development = develop_with_languages(12_750, FOUR_LANGUAGE_MIX);
        simurgh_development.streaks.recent = Some(match ended_days_ago {
            None => RecentStreak::Current(build_weeks(week_count)),
            Some(days) => RecentStreak::Ended {
                weeks: build_weeks(week_count),
                ended_on: simurgh_development.observed_on - Days::new(days),
            },
        });
        collect_opaque_pixel_count(&render_simurgh(
            &simurgh_development,
            SimurghBody::default(),
        ))
    };
    assert!(with_streak(2, None) < with_streak(13, None));
    assert!(with_streak(13, Some(14)) < with_streak(13, None));
    // Long after it ends, the farr has faded away completely.
    assert!(with_streak(13, Some(70)) < with_streak(13, Some(14)));
}

#[test]
fn the_longest_streak_changes_colors_and_a_long_record_adds_sparkle() {
    let render_with_longest = |mix: LanguageMix, longest_weeks: usize| {
        let mut simurgh_development = develop_with_languages(12_750, mix);
        simurgh_development.streaks.longest = build_weeks(longest_weeks);
        render_simurgh(&simurgh_development, SimurghBody::default())
    };
    let normal = render_with_longest(FOUR_LANGUAGE_MIX, 4);
    assert_ne!(render_with_longest(FOUR_LANGUAGE_MIX, 0), normal);
    assert_ne!(render_with_longest(FOUR_LANGUAGE_MIX, 52), normal);
    assert_eq!(
        collect_alphas(&render_with_longest(FOUR_LANGUAGE_MIX, 0)),
        collect_alphas(&normal)
    );
    assert!(
        collect_opaque_pixel_count(&render_with_longest(FOUR_LANGUAGE_MIX, 208))
            > collect_opaque_pixel_count(&normal)
    );
}

#[test]
fn growth_slows_and_keeps_the_whole_bird_inside_a_transparent_frame() {
    let mut widths = Vec::new();
    for contributions in GROWTH_STEPS {
        let simurgh_frame = render_simurgh(
            &develop_with_languages(contributions, FOUR_LANGUAGE_MIX),
            SimurghBody::default(),
        );
        let bounds = measure_opaque_bounds(&simurgh_frame);
        assert!(bounds.left_x >= 4 && bounds.top_y >= 4, "{contributions}");
        assert!(bounds.right_x + 4 < FRAME_WIDTH_PIXELS, "{contributions}");
        assert!(bounds.bottom_y + 4 < FRAME_HEIGHT_PIXELS, "{contributions}");
        widths.push(bounds.right_x - bounds.left_x);
    }
    assert!(
        widths.windows(2).all(|pair| pair[0] < pair[1]),
        "{widths:?}"
    );
    let gains: Vec<u32> = widths.windows(2).map(|pair| pair[1] - pair[0]).collect();
    assert!(gains.last().unwrap() < gains.iter().max().unwrap());
    assert_eq!(
        render_simurgh(
            &develop_with_languages(MAX_GROWTH_CONTRIBUTIONS, FOUR_LANGUAGE_MIX),
            SimurghBody::default()
        ),
        render_simurgh(
            &develop_with_languages(MAX_GROWTH_CONTRIBUTIONS * 5, FOUR_LANGUAGE_MIX),
            SimurghBody::default()
        )
    );
}

#[test]
fn a_chick_is_rounder_and_a_grown_bird_has_more_feathers() {
    let chick = render_simurgh(
        &develop_with_languages(0, FOUR_LANGUAGE_MIX),
        SimurghBody::default(),
    );
    let grown = render_simurgh(
        &develop_with_languages(12_750, FOUR_LANGUAGE_MIX),
        SimurghBody::default(),
    );
    assert!(collect_opaque_pixel_count(&chick) * 3 < collect_opaque_pixel_count(&grown));
    assert!(collect_opaque_colors(&chick).len() < collect_opaque_colors(&grown).len());
}

#[test]
fn every_behavior_keeps_the_whole_bird_inside_the_frame_at_every_size() {
    for contributions in GROWTH_STEPS {
        let simurgh_development = develop_with_languages(contributions, FOUR_LANGUAGE_MIX);
        for behavior in BEHAVIORS {
            for frame_index in (0..frame_count(behavior)).step_by(3) {
                let simurgh_frame = render_flight_frame(
                    &simurgh_development,
                    SimurghBody::default(),
                    behavior,
                    frame_index,
                );
                let bounds = measure_opaque_bounds(&simurgh_frame);
                assert!(
                    bounds.left_x >= 2
                        && bounds.top_y >= 2
                        && bounds.right_x + 2 < FRAME_WIDTH_PIXELS
                        && bounds.bottom_y + 2 < FRAME_HEIGHT_PIXELS,
                    "{contributions} {behavior:?} frame {frame_index}"
                );
            }
        }
    }
}

#[test]
fn only_perching_behaviors_draw_the_branch() {
    let simurgh_development = develop_with_languages(12_750, FOUR_LANGUAGE_MIX);
    let width_of = |behavior: Behavior| {
        let bounds = measure_opaque_bounds(&render_flight_frame(
            &simurgh_development,
            SimurghBody::default(),
            behavior,
            0,
        ));
        bounds.right_x - bounds.left_x
    };
    for behavior in [Behavior::Perch, Behavior::SeedFall, Behavior::Sleep] {
        assert!(width_of(behavior) > 230, "{behavior:?}");
    }
    for behavior in [Behavior::Glide, Behavior::Soar] {
        assert!(width_of(behavior) < 230, "{behavior:?}");
    }
}

#[test]
fn seed_fall_drops_seeds_below_the_bird_that_perching_does_not() {
    let simurgh_development = develop_with_languages(0, FOUR_LANGUAGE_MIX);
    let lowest = |behavior: Behavior| {
        (0..frame_count(behavior))
            .map(|frame_index| {
                measure_opaque_bounds(&render_flight_frame(
                    &simurgh_development,
                    SimurghBody::default(),
                    behavior,
                    frame_index,
                ))
                .bottom_y
            })
            .max()
            .unwrap()
    };
    assert!(lowest(Behavior::SeedFall) > lowest(Behavior::Perch) + 20);
}

#[test]
fn sleep_is_darker_and_bluer_than_perching() {
    let simurgh_development = develop_with_languages(12_750, FOUR_LANGUAGE_MIX);
    let averages = |behavior: Behavior| {
        let simurgh_frame =
            render_flight_frame(&simurgh_development, SimurghBody::default(), behavior, 0);
        let (sums, count) = simurgh_frame
            .as_rgba_bytes()
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel| pixel[3] != 0)
            .fold(([0_u64; 3], 0_u64), |(mut sums, count), pixel| {
                for channel in 0..3 {
                    sums[channel] += u64::from(pixel[channel]);
                }
                (sums, count + 1)
            });
        sums.map(|sum| sum as f64 / count as f64)
    };
    let [sleep_red, sleep_green, sleep_blue] = averages(Behavior::Sleep);
    let [perch_red, perch_green, perch_blue] = averages(Behavior::Perch);
    assert!(sleep_red + sleep_green + sleep_blue < 0.97 * (perch_red + perch_green + perch_blue));
    assert!(sleep_blue / sleep_red > perch_blue / perch_red);
}

#[test]
fn every_loop_is_seamless_and_never_jumps() {
    let simurgh_development = develop_with_languages(12_750, FOUR_LANGUAGE_MIX);
    for behavior in BEHAVIORS {
        let loop_frame_count = frame_count(behavior);
        let frames: Vec<SimurghFrame> = (0..=loop_frame_count)
            .map(|frame_index| {
                render_flight_frame(
                    &simurgh_development,
                    SimurghBody::default(),
                    behavior,
                    frame_index,
                )
            })
            .collect();
        assert_eq!(frames[loop_frame_count as usize], frames[0], "{behavior:?}");
        let steps: Vec<usize> = frames
            .windows(2)
            .map(|pair| count_changed_pixels(&pair[0], &pair[1]))
            .collect();
        let seam = steps[steps.len() - 1];
        let interior_largest = steps[..steps.len() - 1].iter().copied().max().unwrap();
        assert!(
            seam as f64 <= 1.25 * interior_largest as f64,
            "{behavior:?}: the seam changes {seam} pixels against {interior_largest}"
        );
    }
}

#[test]
fn png_output_doubles_each_pixel_and_repeats_exactly() {
    let mut simurgh_development = read_fixture_development();
    simurgh_development.total_contributions = 1_500;
    let simurgh_frame = render_simurgh(&simurgh_development, SimurghBody::default());
    let png_bytes = encode_png(&simurgh_frame).unwrap();
    assert_eq!(
        png_bytes,
        encode_png(&render_simurgh(
            &simurgh_development,
            SimurghBody::default()
        ))
        .unwrap()
    );
    let mut png_reader = png::Decoder::new(Cursor::new(&png_bytes))
        .read_info()
        .unwrap();
    let mut decoded_pixels = vec![0; png_reader.output_buffer_size().unwrap()];
    let frame_info = png_reader.next_frame(&mut decoded_pixels).unwrap();
    assert_eq!((frame_info.width, frame_info.height), (512, 512));
    assert_eq!(frame_info.color_type, png::ColorType::Rgba);
    for (pixel_index, native_channels) in simurgh_frame
        .as_rgba_bytes()
        .as_chunks::<4>()
        .0
        .iter()
        .enumerate()
    {
        let block_left = pixel_index as u32 % FRAME_WIDTH_PIXELS * 2;
        let block_top = pixel_index as u32 / FRAME_WIDTH_PIXELS * 2;
        for (offset_x, offset_y) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            let export_index =
                ((block_top + offset_y) * frame_info.width + block_left + offset_x) as usize * 4;
            assert_eq!(
                &decoded_pixels[export_index..export_index + 4],
                native_channels
            );
        }
    }
}

fn read_fixture_development() -> SimurghDevelopment {
    let account_snapshot =
        serde_json::from_str(include_str!("fixtures/account-with-history.json")).unwrap();
    derive_simurgh_development(&account_snapshot).unwrap()
}

fn develop_with_languages(
    contribution_total: u64,
    language_mix: LanguageMix,
) -> SimurghDevelopment {
    let mut simurgh_development = read_fixture_development();
    simurgh_development.total_contributions = contribution_total;
    simurgh_development.language_shares = language_mix
        .iter()
        .map(|&(language_name, _, language_share)| (language_name.to_owned(), language_share))
        .collect();
    simurgh_development.language_colors = language_mix
        .iter()
        .map(|&(language_name, color_text, _)| {
            (
                language_name.to_owned(),
                color_text.map(|color_text| LanguageColor::try_from(color_text).unwrap()),
            )
        })
        .collect();
    // Normal colors and no farr: a four-week record that ended long ago.
    simurgh_development.streaks.longest = build_weeks(4);
    simurgh_development.streaks.recent = Some(RecentStreak::Ended {
        weeks: build_weeks(4),
        ended_on: simurgh_development.observed_on - Days::new(70),
    });
    simurgh_development
}

/// Fully active weeks; marks depend only on how many there are.
fn build_weeks(week_count: usize) -> Vec<ContributionWeek> {
    let first_monday: NaiveDate = "2024-01-01".parse().unwrap();
    (0..week_count)
        .map(|week_index| ContributionWeek {
            iso_week: (first_monday + Days::new(7 * week_index as u64)).iso_week(),
            active_day_count: 7,
            observed_day_count: 7,
        })
        .collect()
}

fn collect_opaque_pixel_count(simurgh_frame: &SimurghFrame) -> usize {
    simurgh_frame
        .as_rgba_bytes()
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|pixel_channels| pixel_channels[3] != 0)
        .count()
}

fn collect_opaque_colors(simurgh_frame: &SimurghFrame) -> BTreeSet<[u8; 4]> {
    simurgh_frame
        .as_rgba_bytes()
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|pixel_channels| pixel_channels[3] != 0)
        .copied()
        .collect()
}

struct OpaqueBounds {
    left_x: u32,
    top_y: u32,
    right_x: u32,
    bottom_y: u32,
}

fn measure_opaque_bounds(simurgh_frame: &SimurghFrame) -> OpaqueBounds {
    let mut opaque_bounds = OpaqueBounds {
        left_x: FRAME_WIDTH_PIXELS,
        top_y: FRAME_HEIGHT_PIXELS,
        right_x: 0,
        bottom_y: 0,
    };
    for (pixel_index, pixel_channels) in simurgh_frame
        .as_rgba_bytes()
        .as_chunks::<4>()
        .0
        .iter()
        .enumerate()
    {
        if pixel_channels[3] == 0 {
            continue;
        }
        let pixel_x = pixel_index as u32 % FRAME_WIDTH_PIXELS;
        let pixel_y = pixel_index as u32 / FRAME_WIDTH_PIXELS;
        opaque_bounds.left_x = opaque_bounds.left_x.min(pixel_x);
        opaque_bounds.top_y = opaque_bounds.top_y.min(pixel_y);
        opaque_bounds.right_x = opaque_bounds.right_x.max(pixel_x);
        opaque_bounds.bottom_y = opaque_bounds.bottom_y.max(pixel_y);
    }
    assert!(
        opaque_bounds.left_x < opaque_bounds.right_x
            && opaque_bounds.top_y < opaque_bounds.bottom_y
    );
    opaque_bounds
}

fn count_changed_pixels(left: &SimurghFrame, right: &SimurghFrame) -> usize {
    left.as_rgba_bytes()
        .as_chunks::<4>()
        .0
        .iter()
        .zip(right.as_rgba_bytes().as_chunks::<4>().0)
        .filter(|(left_pixel, right_pixel)| left_pixel != right_pixel)
        .count()
}

fn collect_alphas(simurgh_frame: &SimurghFrame) -> Vec<u8> {
    simurgh_frame
        .as_rgba_bytes()
        .as_chunks::<4>()
        .0
        .iter()
        .map(|pixel| pixel[3])
        .collect()
}
