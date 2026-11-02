//! Draw the images the README shows: the growth stages, the behaviors, the farr, and two
//! looping GIFs. The images use one synthetic account, so they never depend on a real
//! person's data and they are reproducible: run it again after changing how the
//! Simurgh looks, then commit the new files.
//!
//! Usage: cargo run --release --example generate-docs -- docs

use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;

use chrono::{Datelike, Days, NaiveDate};
use simurgh::behavior::Behavior;
use simurgh::coat::SimurghBody;
use simurgh::development::{RecentActivity, RecentStreak, SimurghDevelopment, StreakDevelopment};
use simurgh::history::{ContributionWeek, LanguageColor};
use simurgh::motion::frame_count;
use simurgh::output::encode_gif;
use simurgh::render::{FRAME_WIDTH_PIXELS, SimurghFrame, render_flight_frame, render_simurgh};

/// Parchment, readable in both light and dark README themes.
const BACKGROUND: [u8; 3] = [243, 239, 230];
/// The part of the native frame every tile shows: left, top, width, height.
const CROP: [usize; 4] = [8, 6, 240, 236];
const TILE_SCALE: usize = 2;
const GAP: usize = 16;
const LANGUAGES: [(&str, f64, [u8; 3]); 5] = [
    ("Rust", 0.3, [222, 165, 132]),
    ("Python", 0.25, [53, 114, 165]),
    ("TypeScript", 0.2, [49, 120, 198]),
    ("Go", 0.15, [0, 173, 216]),
    ("Ruby", 0.1, [204, 52, 45]),
];
const GROWTH_TOTALS: [u64; 6] = [0, 500, 1_500, 12_750, 50_000, 100_000];

fn main() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let [output_directory] = arguments.as_slice() else {
        eprintln!("Usage: generate-docs DIR");
        std::process::exit(2);
    };
    let output_directory = Path::new(output_directory);
    fs::create_dir_all(output_directory)?;
    let body = SimurghBody::default();

    // Growth stages, from a chick to the largest form.
    let growth_tiles: Vec<SimurghFrame> = GROWTH_TOTALS
        .iter()
        .map(|&total| render_simurgh(&develop(total, 0, 12), body))
        .collect();
    write_sheet(
        &output_directory.join("growth-stages.png"),
        &growth_tiles,
        3,
    )?;

    // One frame of each behavior, picked where the pose reads best.
    let adult = develop(12_750, 9, 30);
    let behavior_tiles: Vec<SimurghFrame> = [
        (Behavior::Glide, 7),
        (Behavior::Soar, 3),
        (Behavior::Perch, 22),
        (Behavior::SeedFall, 33),
        (Behavior::Sleep, 0),
    ]
    .iter()
    .map(|&(behavior, frame_index)| render_flight_frame(&adult, body, behavior, frame_index))
    .collect();
    write_sheet(&output_directory.join("behaviors.png"), &behavior_tiles, 3)?;

    // The farr: no streak, a growing streak, and a long record.
    let farr_tiles: Vec<SimurghFrame> = [(0, 0), (6, 6), (13, 120)]
        .iter()
        .map(|&(current, longest)| render_simurgh(&develop(12_750, current, longest), body))
        .collect();
    write_sheet(&output_directory.join("farr.png"), &farr_tiles, 3)?;

    // Looping GIFs.
    for (behavior, file_name) in [
        (Behavior::Soar, "simurgh.gif"),
        (Behavior::SeedFall, "seed-fall.gif"),
    ] {
        let frames: Vec<SimurghFrame> = (0..frame_count(behavior))
            .map(|frame_index| render_flight_frame(&adult, body, behavior, frame_index))
            .collect();
        fs::write(output_directory.join(file_name), encode_gif(&frames)?)?;
    }
    Ok(())
}

/// A synthetic account with a current streak and a longest streak, in weeks.
fn develop(
    total_contributions: u64,
    current_weeks: usize,
    longest_weeks: usize,
) -> SimurghDevelopment {
    let language_shares: BTreeMap<String, f64> = LANGUAGES
        .iter()
        .map(|&(name, share, _)| (name.to_owned(), share))
        .collect();
    let language_colors = LANGUAGES
        .iter()
        .map(|&(name, _, color)| (name.to_owned(), Some(LanguageColor(color))))
        .collect();
    SimurghDevelopment {
        identity_seed: 7,
        language_shares,
        language_colors,
        total_contributions,
        observed_on: NaiveDate::from_ymd_opt(2026, 10, 31).expect("a valid date"),
        streaks: StreakDevelopment {
            longest: build_weeks(longest_weeks),
            recent: (current_weeks > 0).then(|| RecentStreak::Current(build_weeks(current_weeks))),
        },
        recent_activity: RecentActivity {
            quiet_day_count: 0,
            recent_contributions: 0,
            complete_week_contributions: Vec::new(),
            observation_week_contributions: Vec::new(),
        },
    }
}

fn build_weeks(week_count: usize) -> Vec<ContributionWeek> {
    let first_monday = NaiveDate::from_ymd_opt(2025, 1, 6).expect("a valid date");
    (0..week_count)
        .map(|week_index| ContributionWeek {
            iso_week: (first_monday + Days::new(7 * week_index as u64)).iso_week(),
            active_day_count: 4,
            observed_day_count: 7,
        })
        .collect()
}

/// Lay frames out in a grid on the parchment background, each cropped and doubled.
fn write_sheet(path: &Path, frames: &[SimurghFrame], columns: usize) -> Result<(), Box<dyn Error>> {
    let [crop_left, crop_top, crop_width, crop_height] = CROP;
    let (tile_width, tile_height) = (crop_width * TILE_SCALE, crop_height * TILE_SCALE);
    let rows = frames.len().div_ceil(columns);
    let columns = columns.min(frames.len());
    let width = GAP + columns * (tile_width + GAP);
    let height = GAP + rows * (tile_height + GAP);
    let mut pixels: Vec<u8> = BACKGROUND
        .iter()
        .copied()
        .cycle()
        .take(width * height * 3)
        .collect();
    let side = FRAME_WIDTH_PIXELS as usize;
    for (index, frame) in frames.iter().enumerate() {
        let origin_x = GAP + (index % columns) * (tile_width + GAP);
        let origin_y = GAP + (index / columns) * (tile_height + GAP);
        let native = frame.as_rgba_bytes().as_chunks::<4>().0;
        for tile_y in 0..tile_height {
            for tile_x in 0..tile_width {
                let source = native
                    [(crop_top + tile_y / TILE_SCALE) * side + crop_left + tile_x / TILE_SCALE];
                if source[3] != 0 {
                    let target = ((origin_y + tile_y) * width + origin_x + tile_x) * 3;
                    pixels[target..target + 3].copy_from_slice(&source[..3]);
                }
            }
        }
    }
    let mut encoder = png::Encoder::new(
        BufWriter::new(File::create(path)?),
        width as u32,
        height as u32,
    );
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(&pixels)?;
    Ok(())
}
