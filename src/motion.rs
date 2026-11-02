//! Flight and rest over time: the pose for each frame of a behavior's seamless loop.
//!
//! Every phase in a loop completes a whole number of cycles, so the last frame leads
//! straight back into the first.

use std::f64::consts::TAU;

use crate::behavior::Behavior;

/// Frames per second of every behavior's loop; GIF frame delays derive from it.
pub const FRAMES_PER_SECOND: u32 = 15;

const GLIDE_FRAME_COUNT: u32 = 90;
const SOAR_FRAME_COUNT: u32 = 60;
const PERCH_FRAME_COUNT: u32 = 90;
const SEED_FALL_FRAME_COUNT: u32 = 90;
const SLEEP_FRAME_COUNT: u32 = 120;

/// Wingbeats per loop while gliding and soaring.
const GLIDE_BEATS_PER_LOOP: f64 = 3.0;
const SOAR_BEATS_PER_LOOP: f64 = 5.0;
/// How many times the Tree of All Seeds is shaken, and seeds fall, per loop.
const SEED_CYCLES_PER_LOOP: f64 = 3.0;
/// The wing's tip lags its root by this many radians of beat phase.
const WING_LAG_RADIANS: f64 = 0.7;

/// Where the Simurgh is and how every moving part is held in one frame. Offsets and
/// angles are relative to the still pose; the renderer owns anatomy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlightPose {
    /// Shift of the whole bird from the frame center, in logical pixels.
    pub offset: [f64; 2],
    /// Wing height: 1 fully raised, 0 level behind the body, -1 fully lowered.
    pub wing_lift: f64,
    /// How far the wing is open, from 0 folded against the body through 1 fully spread.
    pub wing_spread: f64,
    /// Bend of the feather tips behind the wing's motion, in radians.
    pub wing_bend_radians: f64,
    /// Phase of the wave running down the tail streamers.
    pub tail_phase_radians: f64,
    /// Size of the tail wave, from 0 hanging still through 1 sweeping wide.
    pub tail_strength: f64,
    /// Whether the bird sits on the branch of the Tree of All Seeds.
    pub perched: bool,
    /// How far the head is tucked under the wing, from 0 up through 1 fully tucked.
    pub head_tuck: f64,
    /// Where the head turns while perched, from -1 looking back through 1 looking ahead.
    pub head_turn: f64,
    /// How open the eye is, from 0 shut through 1 open.
    pub eye_open: f64,
    /// Where the falling seeds are in their cycle, from 0 through 1; `None` when none fall.
    pub seed_phase: Option<f64>,
    /// Phase of the plumage sparkle.
    pub sparkle_phase_radians: f64,
    /// How far the bird is dimmed and tinted blue for sleep, from 0 through 1.
    pub dimming: f64,
}

/// The fixed pose of the still image: wings lifted, tail flowing.
pub(crate) const STILL_POSE: FlightPose = FlightPose {
    offset: [0.0, 0.0],
    wing_lift: 0.45,
    wing_spread: 0.95,
    wing_bend_radians: 0.18,
    tail_phase_radians: 0.9,
    tail_strength: 0.6,
    perched: false,
    head_tuck: 0.0,
    head_turn: 1.0,
    eye_open: 1.0,
    seed_phase: None,
    sparkle_phase_radians: 0.0,
    dimming: 0.0,
};

/// Frames in a behavior's loop at [`FRAMES_PER_SECOND`].
pub fn frame_count(behavior: Behavior) -> u32 {
    match behavior {
        Behavior::Glide => GLIDE_FRAME_COUNT,
        Behavior::Soar => SOAR_FRAME_COUNT,
        Behavior::Perch => PERCH_FRAME_COUNT,
        Behavior::SeedFall => SEED_FALL_FRAME_COUNT,
        Behavior::Sleep => SLEEP_FRAME_COUNT,
    }
}

/// The pose for a frame of a behavior's loop; frame numbers wrap around the loop, so
/// frame `frame_count(behavior)` is frame zero again.
///
/// Gliding beats slowly while the body rises and settles behind each stroke. Soaring
/// beats fast and sweeps the tail wide. Perching sits folded on the branch, turning
/// its head and breathing. Seed fall shakes the branch with quick wing flicks while
/// seeds drop. Sleeping tucks the head, shuts the eye, and barely breathes, dimmed
/// and tinted blue. In every behavior the tail wave and the sparkle complete whole
/// cycles per loop.
pub fn calculate_flight_pose(behavior: Behavior, frame_index: u32) -> FlightPose {
    let loop_frame_count = frame_count(behavior);
    let loop_position = f64::from(frame_index % loop_frame_count) / f64::from(loop_frame_count);
    let cycle = |cycles_per_loop: f64| TAU * cycles_per_loop * loop_position;
    let sparkle_phase_radians = cycle(2.0);
    match behavior {
        Behavior::Glide => {
            let beat = cycle(GLIDE_BEATS_PER_LOOP);
            FlightPose {
                offset: [2.5 * cycle(1.0).sin(), -2.5 * (beat - 0.8).sin()],
                wing_lift: 0.15 + 0.85 * beat.sin(),
                wing_spread: 0.95,
                wing_bend_radians: 0.32 * (beat - WING_LAG_RADIANS).cos(),
                tail_phase_radians: cycle(2.0),
                tail_strength: 0.6,
                sparkle_phase_radians,
                ..STILL_POSE
            }
        }
        Behavior::Soar => {
            let beat = cycle(SOAR_BEATS_PER_LOOP);
            FlightPose {
                offset: [3.0 * cycle(1.0).sin(), -3.5 * (beat - 0.9).sin()],
                wing_lift: 0.1 + 0.95 * beat.sin(),
                wing_spread: 1.0,
                wing_bend_radians: 0.45 * (beat - WING_LAG_RADIANS).cos(),
                tail_phase_radians: cycle(5.0),
                tail_strength: 1.0,
                sparkle_phase_radians,
                ..STILL_POSE
            }
        }
        Behavior::Perch => {
            let look = cycle(1.0).sin();
            FlightPose {
                offset: [0.0, 0.6 * cycle(3.0).sin()],
                wing_lift: 0.0,
                wing_spread: 0.08 + 0.05 * cycle(3.0).sin().max(0.0),
                wing_bend_radians: 0.0,
                tail_phase_radians: cycle(1.0),
                tail_strength: 0.18,
                perched: true,
                head_turn: look,
                sparkle_phase_radians,
                ..STILL_POSE
            }
        }
        Behavior::SeedFall => {
            let shake = cycle(SEED_CYCLES_PER_LOOP);
            FlightPose {
                offset: [0.8 * shake.sin(), 0.6 * (shake * 2.0).sin()],
                wing_lift: 0.5 * shake.sin(),
                wing_spread: 0.3 + 0.2 * shake.sin().abs(),
                wing_bend_radians: 0.2 * shake.cos(),
                tail_phase_radians: cycle(3.0),
                tail_strength: 0.3,
                perched: true,
                head_turn: 1.0,
                seed_phase: Some((SEED_CYCLES_PER_LOOP * loop_position).fract()),
                sparkle_phase_radians,
                ..STILL_POSE
            }
        }
        Behavior::Sleep => FlightPose {
            offset: [0.0, 0.8 * cycle(2.0).sin()],
            wing_lift: 0.0,
            wing_spread: 0.06,
            wing_bend_radians: 0.0,
            tail_phase_radians: cycle(1.0),
            tail_strength: 0.06,
            perched: true,
            head_tuck: 1.0,
            head_turn: 0.0,
            eye_open: 0.0,
            sparkle_phase_radians,
            dimming: 0.4,
            ..STILL_POSE
        },
    }
}
