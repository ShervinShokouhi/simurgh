use simurgh::behavior::Behavior;
use simurgh::motion::{FRAMES_PER_SECOND, FlightPose, calculate_flight_pose, frame_count};

type PoseReading = (&'static str, fn(&FlightPose) -> f64);

const BEHAVIORS: [Behavior; 5] = [
    Behavior::Glide,
    Behavior::Soar,
    Behavior::Perch,
    Behavior::SeedFall,
    Behavior::Sleep,
];

fn loop_poses(behavior: Behavior) -> Vec<FlightPose> {
    (0..frame_count(behavior))
        .map(|frame_index| calculate_flight_pose(behavior, frame_index))
        .collect()
}

#[test]
fn every_loop_is_a_whole_number_of_seconds_or_close_to_it() {
    assert_eq!(FRAMES_PER_SECOND, 15);
    for behavior in BEHAVIORS {
        assert!(frame_count(behavior) >= 60, "{behavior:?}");
        assert_eq!(frame_count(behavior) % FRAMES_PER_SECOND, 0, "{behavior:?}");
    }
}

#[test]
fn frame_numbers_wrap_around_the_loop() {
    for behavior in BEHAVIORS {
        let loop_frame_count = frame_count(behavior);
        for frame_index in [0, 7, loop_frame_count - 1] {
            assert_eq!(
                calculate_flight_pose(behavior, frame_index),
                calculate_flight_pose(behavior, frame_index + loop_frame_count),
                "{behavior:?} frame {frame_index}"
            );
        }
    }
}

#[test]
fn the_last_frame_leads_back_into_the_first_as_smoothly_as_any_other_step() {
    let continuous: [PoseReading; 6] = [
        ("wing lift", |pose| pose.wing_lift),
        ("wing spread", |pose| pose.wing_spread),
        ("wing bend", |pose| pose.wing_bend_radians),
        ("tail strength", |pose| pose.tail_strength),
        ("horizontal offset", |pose| pose.offset[0]),
        ("vertical offset", |pose| pose.offset[1]),
    ];
    for behavior in BEHAVIORS {
        let poses = loop_poses(behavior);
        for (name, read) in continuous {
            let steps: Vec<f64> = (0..poses.len())
                .map(|index| (read(&poses[(index + 1) % poses.len()]) - read(&poses[index])).abs())
                .collect();
            let seam = steps[steps.len() - 1];
            let interior_largest = steps[..steps.len() - 1].iter().copied().fold(0.0, f64::max);
            assert!(
                seam <= interior_largest + 1e-9,
                "{behavior:?} {name} jumps at the seam: {seam} against {interior_largest}"
            );
        }
    }
}

#[test]
fn flying_behaviors_beat_their_wings_and_perched_ones_keep_them_folded() {
    for behavior in [Behavior::Glide, Behavior::Soar] {
        let poses = loop_poses(behavior);
        assert!(poses.iter().all(|pose| !pose.perched), "{behavior:?}");
        assert!(
            poses.iter().all(|pose| pose.wing_spread > 0.9),
            "{behavior:?}"
        );
        let highest = poses
            .iter()
            .map(|pose| pose.wing_lift)
            .fold(f64::MIN, f64::max);
        let lowest = poses
            .iter()
            .map(|pose| pose.wing_lift)
            .fold(f64::MAX, f64::min);
        assert!(
            highest > 0.7 && lowest < -0.5,
            "{behavior:?} {lowest}..{highest}"
        );
    }
    for behavior in [Behavior::Perch, Behavior::SeedFall, Behavior::Sleep] {
        let poses = loop_poses(behavior);
        assert!(poses.iter().all(|pose| pose.perched), "{behavior:?}");
    }
    assert!(
        loop_poses(Behavior::Perch)
            .iter()
            .all(|pose| pose.wing_spread < 0.2)
    );
}

#[test]
fn soaring_beats_faster_and_sweeps_the_tail_wider_than_gliding() {
    let beats = |behavior: Behavior| {
        let poses = loop_poses(behavior);
        let seconds = f64::from(frame_count(behavior)) / f64::from(FRAMES_PER_SECOND);
        let rising = poses
            .iter()
            .enumerate()
            .filter(|(index, pose)| {
                pose.wing_lift < 0.0 && poses[(index + 1) % poses.len()].wing_lift >= 0.0
            })
            .count();
        rising as f64 / seconds
    };
    assert!(beats(Behavior::Soar) > 1.5 * beats(Behavior::Glide));
    assert!(
        calculate_flight_pose(Behavior::Soar, 0).tail_strength
            > calculate_flight_pose(Behavior::Glide, 0).tail_strength
    );
}

#[test]
fn only_seed_fall_drops_seeds_and_they_cycle_through_the_loop() {
    for behavior in BEHAVIORS {
        let phases: Vec<Option<f64>> = loop_poses(behavior)
            .iter()
            .map(|pose| pose.seed_phase)
            .collect();
        if behavior == Behavior::SeedFall {
            let phases: Vec<f64> = phases.into_iter().map(|phase| phase.unwrap()).collect();
            assert!(phases.iter().all(|phase| (0.0..1.0).contains(phase)));
            let wraps = phases.windows(2).filter(|pair| pair[1] < pair[0]).count();
            assert_eq!(wraps, 2, "three cycles wrap twice inside the loop");
        } else {
            assert!(phases.iter().all(Option::is_none), "{behavior:?}");
        }
    }
}

#[test]
fn sleep_tucks_the_head_shuts_the_eye_and_dims_the_bird() {
    for pose in loop_poses(Behavior::Sleep) {
        assert_eq!(pose.head_tuck, 1.0);
        assert_eq!(pose.eye_open, 0.0);
        assert!(pose.dimming > 0.3);
    }
    for behavior in [
        Behavior::Glide,
        Behavior::Soar,
        Behavior::Perch,
        Behavior::SeedFall,
    ] {
        for pose in loop_poses(behavior) {
            assert_eq!(pose.head_tuck, 0.0, "{behavior:?}");
            assert_eq!(pose.eye_open, 1.0, "{behavior:?}");
            assert_eq!(pose.dimming, 0.0, "{behavior:?}");
        }
    }
}
