use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use chrono::Days;
use simurgh::behavior::{Behavior, select_behavior};
use simurgh::coat::SimurghBody;
use simurgh::development::{SimurghDevelopment, derive_simurgh_development};
use simurgh::history::{AccountSnapshot, ContributionDay};
use simurgh::motion::frame_count;
use simurgh::output::{encode_gif, encode_png};
use simurgh::render::{SimurghFrame, render_flight_frame, render_simurgh};

const HISTORY_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/account-with-history.json"
);

#[test]
fn render_writes_the_days_loop_still_and_preview_as_the_renderer_draws_them() {
    // Two quiet weeks make the Simurgh perch, so the loop can't pass as the fallback glide.
    let mut account_snapshot = read_fixture_snapshot();
    let contribution_history = account_snapshot.contribution_history.as_mut().unwrap();
    for day_offset in 1..=14 {
        contribution_history.days.push(ContributionDay {
            date: contribution_history.observed_on + Days::new(day_offset),
            contribution_count: 0,
        });
    }
    contribution_history.observed_on = contribution_history.days.last().unwrap().date;
    let scratch_directory = fresh_directory("render-default");
    let snapshot_path = scratch_directory.join("snapshot.json");
    fs::write(
        &snapshot_path,
        serde_json::to_string(&account_snapshot).unwrap(),
    )
    .unwrap();
    let output_directory = scratch_directory.join("simurgh");
    let command_output = run_simurgh(&[
        "render",
        snapshot_path.to_str().unwrap(),
        "--output-dir",
        output_directory.to_str().unwrap(),
    ]);
    assert_eq!(
        command_output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&command_output.stderr)
    );

    let simurgh_development = derive_simurgh_development(&account_snapshot).unwrap();
    let behavior = select_behavior(&simurgh_development);
    assert_eq!(behavior, Behavior::Perch);
    let flight_frames: Vec<SimurghFrame> = (0..frame_count(behavior))
        .map(|frame_index| {
            render_flight_frame(
                &simurgh_development,
                SimurghBody::Zarrin,
                behavior,
                frame_index,
            )
        })
        .collect();
    assert_eq!(
        fs::read(output_directory.join("simurgh.gif")).unwrap(),
        encode_gif(&flight_frames).unwrap()
    );
    assert_eq!(
        fs::read(output_directory.join("simurgh.png")).unwrap(),
        encode_png(&render_simurgh(&simurgh_development, SimurghBody::Zarrin)).unwrap()
    );
    let preview_html = fs::read_to_string(output_directory.join("preview.html")).unwrap();
    assert!(preview_html.contains(r#"src="simurgh.gif""#));
    assert!(preview_html.contains(r#"src="simurgh.png""#));
    assert!(!preview_html.contains("<script"));
    let summary_line = String::from_utf8_lossy(&command_output.stdout);
    let observation_date = simurgh_development.observed_on.to_string();
    assert!(summary_line.contains(&observation_date));
    assert!(preview_html.contains(&observation_date));
}

#[test]
fn the_body_option_redraws_the_koi_in_that_body() {
    let output_directory = fresh_directory("render-shab");
    let command_output = run_simurgh(&[
        "render",
        HISTORY_FIXTURE,
        "--output-dir",
        output_directory.to_str().unwrap(),
        "--body",
        "shab",
    ]);
    assert_eq!(command_output.status.code(), Some(0));
    let simurgh_development = read_fixture_development();
    let shab_png = encode_png(&render_simurgh(&simurgh_development, SimurghBody::Shab)).unwrap();
    assert_ne!(
        shab_png,
        encode_png(&render_simurgh(&simurgh_development, SimurghBody::Zarrin)).unwrap()
    );
    assert_eq!(
        fs::read(output_directory.join("simurgh.png")).unwrap(),
        shab_png
    );
    assert!(String::from_utf8_lossy(&command_output.stdout).contains("shab body"));
}

#[test]
fn a_failed_render_exits_one_and_keeps_the_previous_files() {
    let output_directory = fresh_directory("render-failure");
    for file_name in ["simurgh.gif", "simurgh.png", "preview.html"] {
        fs::write(
            output_directory.join(file_name),
            format!("previous {file_name}"),
        )
        .unwrap();
    }
    let command_output = run_simurgh(&[
        "render",
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/account.json"),
        "--output-dir",
        output_directory.to_str().unwrap(),
    ]);
    assert_eq!(command_output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&command_output.stderr).starts_with("simurgh: "));
    for file_name in ["simurgh.gif", "simurgh.png", "preview.html"] {
        assert_eq!(
            fs::read_to_string(output_directory.join(file_name)).unwrap(),
            format!("previous {file_name}")
        );
    }
    assert_eq!(fs::read_dir(&output_directory).unwrap().count(), 3);
}

#[test]
fn render_usage_errors_exit_two_and_help_exits_zero() {
    for invalid_arguments in [
        &["render", HISTORY_FIXTURE][..],
        &["render", "--output-dir", "simurgh"],
        &[
            "render",
            HISTORY_FIXTURE,
            "--output-dir",
            "simurgh",
            "--body",
            "red",
        ],
    ] {
        let command_output = run_simurgh(invalid_arguments);
        assert_eq!(
            command_output.status.code(),
            Some(2),
            "{invalid_arguments:?}"
        );
        assert!(String::from_utf8_lossy(&command_output.stderr).starts_with("Usage:"));
    }
    let help_output = run_simurgh(&["render", "--help"]);
    assert_eq!(help_output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&help_output.stdout).contains("arghavan"));
}

#[test]
fn version_prints_the_manifest_version_and_exits_zero() {
    let version_output = run_simurgh(&["--version"]);
    assert_eq!(version_output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&version_output.stdout),
        format!("simurgh {}\n", env!("CARGO_PKG_VERSION"))
    );
}

fn read_fixture_snapshot() -> AccountSnapshot {
    serde_json::from_str(&fs::read_to_string(HISTORY_FIXTURE).unwrap()).unwrap()
}

fn read_fixture_development() -> SimurghDevelopment {
    derive_simurgh_development(&read_fixture_snapshot()).unwrap()
}

/// An empty directory of the given name under Cargo's scratch space for tests.
fn fresh_directory(directory_name: &str) -> PathBuf {
    let directory_path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(directory_name);
    let _ = fs::remove_dir_all(&directory_path);
    fs::create_dir_all(&directory_path).unwrap();
    directory_path
}

fn run_simurgh(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_simurgh"))
        .args(arguments)
        .output()
        .unwrap()
}
