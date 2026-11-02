//! A pixel-art Simurgh drawn from curves and feather strokes, with transparent
//! surroundings.
//!
//! The bird is drawn on a [`LOGICAL_SIDE_PIXELS`]-pixel square canvas and doubled to the
//! [`FRAME_WIDTH_PIXELS`] native frame, so every drawn pixel is a clean two-pixel block.
//! Its shapes follow the data: contribution totals set its size and anatomy through
//! [`crate::development::calculate_growth_fraction`], from a round-headed chick with a
//! short tail to a crowned bird whose tail streamers each end in an eye. A young
//! Simurgh has a larger head, shorter wings, and fewer feathers; after
//! [`crate::development::ADULT_CONTRIBUTIONS`] the tail and crest keep lengthening, and
//! the maximum form keeps its anatomy at higher totals.
//!
//! Plumage lives in [`crate::coat`]: the body palette, and the language colors on the
//! tail. The streak marks from [`crate::markings`] show as the golden farr, a halo of
//! light dots, gold on the wing tips and tail rims, and as sparkle. Poses come from
//! [`crate::motion`]. Everything here is deterministic: it reads no clock, performs no
//! I/O, and mutates no input.

use std::f64::consts::{PI, TAU};

use crate::behavior::Behavior;
use crate::coat::{Coat, Ramp, Rgba, SimurghBody, dress, outline, splitmix64};
use crate::development::{
    ADULT_CONTRIBUTIONS, MAX_GROWTH_CONTRIBUTIONS, SimurghDevelopment, YOUNG_SHAPE_CONTRIBUTIONS,
    calculate_growth_fraction,
};
use crate::motion::{FlightPose, STILL_POSE, calculate_flight_pose};

/// Width of the native canvas every Simurgh is drawn on, at every growth level.
pub const FRAME_WIDTH_PIXELS: u32 = 256;
/// Height of the native canvas; the canvas is square.
pub const FRAME_HEIGHT_PIXELS: u32 = FRAME_WIDTH_PIXELS;
/// Side of the logical canvas that the frame doubles.
pub const LOGICAL_SIDE_PIXELS: u32 = 128;

const SIDE: usize = LOGICAL_SIDE_PIXELS as usize;
const UPSCALE: usize = (FRAME_WIDTH_PIXELS / LOGICAL_SIDE_PIXELS) as usize;
/// Direction toward the light, up and to the left.
const TOWARD_LIGHT: Vec2 = [-0.45, -0.89];
const BAYER: [[f64; 4]; 4] = [
    [0.0, 8.0, 2.0, 10.0],
    [12.0, 4.0, 14.0, 6.0],
    [3.0, 11.0, 1.0, 9.0],
    [15.0, 7.0, 13.0, 5.0],
];
/// The branch of the Tree of All Seeds the bird perches on.
const BRANCH_Y: f64 = 70.0;
const BRANCH_X_RANGE: [f64; 2] = [4.0, 124.0];
const FLIGHT_ANCHOR: Vec2 = [82.0, 50.0];
const PERCH_ANCHOR_X: f64 = 82.0;
const BRANCH_RAMP: Ramp = [
    [44, 28, 38, 255],
    [88, 52, 48, 255],
    [140, 90, 60, 255],
    [190, 140, 90, 255],
];
const LEAF_RAMP: Ramp = [
    [24, 84, 66, 255],
    [46, 140, 88, 255],
    [104, 200, 120, 255],
    [176, 236, 160, 255],
];

type Vec2 = [f64; 2];

/// A renderer-owned, fixed-size frame with row-major RGBA pixels.
#[derive(Debug, Eq, PartialEq)]
pub struct SimurghFrame {
    rgba_pixels: Vec<[u8; 4]>,
}

impl SimurghFrame {
    /// The pixels as bytes, row by row from the top left, four channels per pixel: red,
    /// green, blue, alpha. The length is the canvas area times four.
    pub fn as_rgba_bytes(&self) -> &[u8] {
        self.rgba_pixels.as_flattened()
    }

    /// Build a frame from chosen pixels, for encoder tests that need colors no Simurgh has.
    #[cfg(test)]
    pub(crate) fn from_rgba_pixels(rgba_pixels: Vec<[u8; 4]>) -> Self {
        Self { rgba_pixels }
    }
}

/// Render the Simurgh in a centered, fixed flying pose, with its plumage on the caller's
/// chosen body and its tail in its languages' colors. Contribution totals set its size
/// and how far its colors have come in. The account seed varies the details of the
/// feathers, so growth does not reroll them. Streak marks follow the development's
/// streaks as of its observation date. Identical inputs produce identical pixels
/// without I/O, clock reads, or input mutation.
pub fn render_simurgh(
    simurgh_development: &SimurghDevelopment,
    simurgh_body: SimurghBody,
) -> SimurghFrame {
    render_pose(simurgh_development, simurgh_body, &STILL_POSE)
}

/// Render one frame of a behavior's seamless loop, colored like [`render_simurgh`], in
/// the pose [`calculate_flight_pose`] gives; frame numbers wrap around the loop of
/// [`crate::motion::frame_count`] frames. The view never follows the bird, so it never
/// shakes. Identical inputs produce identical pixels without I/O, clock reads, or input
/// mutation.
pub fn render_flight_frame(
    simurgh_development: &SimurghDevelopment,
    simurgh_body: SimurghBody,
    behavior: Behavior,
    frame_index: u32,
) -> SimurghFrame {
    let flight_pose = calculate_flight_pose(behavior, frame_index);
    render_pose(simurgh_development, simurgh_body, &flight_pose)
}

fn render_pose(
    simurgh_development: &SimurghDevelopment,
    simurgh_body: SimurghBody,
    flight_pose: &FlightPose,
) -> SimurghFrame {
    let anatomy = Anatomy::derive(simurgh_development.total_contributions);
    let coat = dress(simurgh_development, simurgh_body);
    let seed = simurgh_development.identity_seed;
    let mut canvas = Canvas::new();
    let mut shine_points: Vec<Vec2> = Vec::new();
    let skeleton = Skeleton::build(&anatomy, flight_pose);

    if flight_pose.perched {
        draw_branch(&mut canvas, seed);
    }
    draw_wing(
        &mut canvas,
        &skeleton,
        &anatomy,
        flight_pose,
        &coat,
        WingSide::Far,
        &mut shine_points,
    );
    draw_tail(
        &mut canvas,
        &skeleton,
        &anatomy,
        flight_pose,
        &coat,
        seed,
        &mut shine_points,
    );
    draw_legs(&mut canvas, &skeleton, &anatomy, flight_pose, &coat);
    draw_body(&mut canvas, &skeleton, &anatomy, &coat);
    draw_neck_and_head(&mut canvas, &skeleton, &anatomy, flight_pose, &coat);
    draw_wing(
        &mut canvas,
        &skeleton,
        &anatomy,
        flight_pose,
        &coat,
        WingSide::Near,
        &mut shine_points,
    );
    if flight_pose.perched {
        draw_seeds(&mut canvas, flight_pose, &coat, seed);
    }
    canvas.add_outline();
    draw_halo(&mut canvas, &skeleton, &anatomy, &coat, seed, flight_pose);
    draw_sparkles(&mut canvas, &coat, &shine_points, seed, flight_pose);
    if flight_pose.dimming > 0.0 {
        canvas.dim(flight_pose.dimming);
    }
    canvas.into_frame()
}

/// Proportions that follow the contribution total.
struct Anatomy {
    scale: f64,
    /// One for a chick, falling to zero by the young-shape total.
    youth: f64,
    body_half_length: f64,
    body_half_height: f64,
    neck_length: f64,
    head_radius: f64,
    wing_count: usize,
    wing_length: f64,
    tail_count: usize,
    tail_length: f64,
    tail_eyes: bool,
    crest_count: usize,
    crest_length: f64,
}

impl Anatomy {
    fn derive(total_contributions: u64) -> Self {
        let growth = calculate_growth_fraction(total_contributions);
        let youth = 1.0 - (total_contributions as f64 / YOUNG_SHAPE_CONTRIBUTIONS as f64).min(1.0);
        let maturity = (total_contributions as f64 / ADULT_CONTRIBUTIONS as f64).min(1.0);
        let beyond = (total_contributions.saturating_sub(ADULT_CONTRIBUTIONS) as f64
            / (MAX_GROWTH_CONTRIBUTIONS - ADULT_CONTRIBUTIONS) as f64)
            .min(1.0);
        let scale = 0.66 + 0.36 * growth;
        let wing_progress = 1.0 - youth * 0.45;
        Self {
            scale,
            youth,
            body_half_length: 14.0 * scale,
            body_half_height: 9.5 * scale,
            neck_length: 15.0 * scale * (1.0 - 0.25 * youth),
            head_radius: 5.6 * scale * (1.0 + 0.45 * youth),
            wing_count: 4 + (4.0 * maturity.powf(0.6)).round() as usize,
            wing_length: 32.0 * scale * wing_progress,
            tail_count: 3 + (6.0 * maturity.powf(0.7)).round() as usize,
            tail_length: 58.0 * scale * (0.34 + 0.66 * maturity.powf(0.8)) * (1.0 + 0.05 * beyond),
            tail_eyes: total_contributions >= YOUNG_SHAPE_CONTRIBUTIONS,
            crest_count: 2 + (3.0 * maturity).round() as usize,
            crest_length: (4.0 + 7.0 * maturity.powf(0.7) + 3.0 * beyond) * scale,
        }
    }
}

/// Where each part of the body sits in a pose.
struct Skeleton {
    center: Vec2,
    axis_angle: f64,
    axis: Vec2,
    up: Vec2,
    neck_start: Vec2,
    neck_control: Vec2,
    head: Vec2,
    beak_angle: f64,
    shoulder: Vec2,
    tail_root: Vec2,
}

impl Skeleton {
    fn build(anatomy: &Anatomy, flight_pose: &FlightPose) -> Self {
        let scale = anatomy.scale;
        let axis_angle = if flight_pose.perched {
            -0.85
        } else {
            -0.15 - 0.06 * flight_pose.wing_lift
        };
        let axis = polar(axis_angle, 1.0);
        let up = [axis_angle.sin(), -axis_angle.cos()];
        let base = if flight_pose.perched {
            [PERCH_ANCHOR_X, BRANCH_Y - 12.5 * scale - 2.0]
        } else {
            FLIGHT_ANCHOR
        };
        let center = [
            base[0] + flight_pose.offset[0],
            base[1] + flight_pose.offset[1],
        ];
        let (half_length, half_height) = (anatomy.body_half_length, anatomy.body_half_height);
        let neck_start = add(
            add(center, scale_by(axis, half_length * 0.72)),
            scale_by(up, half_height * 0.35),
        );
        let neck_angle = if flight_pose.perched {
            axis_angle - 0.55
        } else {
            axis_angle - 0.2
        };
        let tuck = flight_pose.head_tuck;
        let open_head = add(neck_start, polar(neck_angle, anatomy.neck_length));
        let tucked_head = add(
            add(center, scale_by(up, half_height * 0.95)),
            scale_by(axis, half_length * 0.25),
        );
        let turned_head = add(
            open_head,
            scale_by(axis, 1.2 * (flight_pose.head_turn - 1.0) * (1.0 - tuck)),
        );
        let head = lerp_point(turned_head, tucked_head, tuck);
        let open_control = add(
            add(neck_start, polar(axis_angle, anatomy.neck_length * 0.55)),
            scale_by(up, anatomy.neck_length * 0.1),
        );
        let tucked_control = add(neck_start, scale_by(up, half_height * 1.6));
        let neck_control = lerp_point(open_control, tucked_control, tuck);
        let open_beak = if flight_pose.perched {
            0.0
        } else {
            axis_angle + 0.05
        } + 0.3 * (1.0 - flight_pose.head_turn);
        let beak_angle = open_beak + (PI - 0.5 - open_beak) * tuck;
        let shoulder = add(
            add(center, scale_by(axis, half_length * 0.3)),
            scale_by(up, half_height * 0.55),
        );
        let tail_root = add(
            center,
            add(
                scale_by(axis, -half_length * 0.92),
                scale_by(up, -half_height * 0.05),
            ),
        );
        Self {
            center,
            axis_angle,
            axis,
            up,
            neck_start,
            neck_control,
            head,
            beak_angle,
            shoulder,
            tail_root,
        }
    }
}

// ---------------------------------------------------------------------------
// Canvas

struct Canvas {
    pixels: Vec<Option<Rgba>>,
}

/// What a stroke knows about the spot it is painting.
struct Sample {
    /// Across the stroke, from -1 on one edge through 0 on the spine to 1 on the other.
    across: f64,
    /// Along the stroke, from 0 at its start to 1 at its end.
    along: f64,
    normal: Vec2,
    cell: [i32; 2],
}

impl Canvas {
    fn new() -> Self {
        Self {
            pixels: vec![None; SIDE * SIDE],
        }
    }

    fn put(&mut self, x: i32, y: i32, color: Rgba) {
        if (0..SIDE as i32).contains(&x) && (0..SIDE as i32).contains(&y) {
            self.pixels[y as usize * SIDE + x as usize] = Some(color);
        }
    }

    fn get(&self, x: i32, y: i32) -> Option<Rgba> {
        if (0..SIDE as i32).contains(&x) && (0..SIDE as i32).contains(&y) {
            self.pixels[y as usize * SIDE + x as usize]
        } else {
            None
        }
    }

    /// Paint a stroke along a path, as wide as `width` says at each fraction of its
    /// length, in the color `paint` gives for each spot, if any.
    fn stroke(
        &mut self,
        path: &[Vec2],
        width: impl Fn(f64) -> f64,
        paint: impl Fn(&Sample) -> Option<Rgba>,
    ) {
        const STEP: f64 = 0.35;
        let mut distances = vec![0.0];
        for pair in path.windows(2) {
            let last = *distances.last().expect("distances start with zero");
            distances.push(last + distance(pair[0], pair[1]));
        }
        let total = *distances.last().expect("distances start with zero");
        if total <= 0.0 {
            return;
        }
        let steps = (total / STEP).ceil() as usize;
        for step in 0..=steps {
            let traveled = total * step as f64 / steps as f64;
            let (point, tangent) = sample_path(path, &distances, traveled);
            let along = traveled / total;
            let normal = [-tangent[1], tangent[0]];
            let half_width = width(along) / 2.0;
            let lateral_steps = if half_width < 0.3 {
                0
            } else {
                ((half_width * 2.0) / STEP).ceil() as i32
            };
            for lateral in 0..=lateral_steps {
                let across = if lateral_steps == 0 {
                    0.0
                } else {
                    -1.0 + 2.0 * f64::from(lateral) / f64::from(lateral_steps)
                };
                let spot = add(point, scale_by(normal, across * half_width));
                let cell = [spot[0].floor() as i32, spot[1].floor() as i32];
                let sample = Sample {
                    across,
                    along,
                    normal,
                    cell,
                };
                if let Some(color) = paint(&sample) {
                    self.put(cell[0], cell[1], color);
                }
            }
        }
    }

    /// Paint a shaded disc with the light from the upper left.
    fn disc(&mut self, center: Vec2, radius: f64, ramp: &[Rgba]) {
        let reach = radius.ceil() as i32 + 1;
        let (center_x, center_y) = (center[0].floor() as i32, center[1].floor() as i32);
        for cell_y in center_y - reach..=center_y + reach {
            for cell_x in center_x - reach..=center_x + reach {
                let offset = [
                    f64::from(cell_x) + 0.5 - center[0],
                    f64::from(cell_y) + 0.5 - center[1],
                ];
                if dot(offset, offset) <= radius * radius {
                    let facing = scale_by(offset, 1.0 / radius);
                    let depth = (1.0 - dot(facing, facing)).max(0.0).sqrt();
                    let tone = 0.5 + 0.5 * (0.8 * dot(facing, TOWARD_LIGHT) + 0.5 * depth - 0.2);
                    self.put(cell_x, cell_y, pick(ramp, tone));
                }
            }
        }
    }

    /// Surround every shape with a one-pixel dark outline.
    fn add_outline(&mut self) {
        let mut outlined = self.pixels.clone();
        for cell_y in 0..SIDE as i32 {
            for cell_x in 0..SIDE as i32 {
                if self.get(cell_x, cell_y).is_none()
                    && [(1, 0), (-1, 0), (0, 1), (0, -1)]
                        .iter()
                        .any(|(step_x, step_y)| {
                            self.get(cell_x + step_x, cell_y + step_y).is_some()
                        })
                {
                    outlined[cell_y as usize * SIDE + cell_x as usize] = Some(outline());
                }
            }
        }
        self.pixels = outlined;
    }

    /// Tint toward night blue and darken, for sleep.
    fn dim(&mut self, amount: f64) {
        const NIGHT: [f64; 3] = [52.0, 74.0, 140.0];
        for pixel in self.pixels.iter_mut().flatten() {
            for channel in 0..3 {
                let tinted = f64::from(pixel[channel]) * (1.0 - amount * 0.45)
                    + NIGHT[channel] * amount * 0.45;
                pixel[channel] = (tinted * (1.0 - 0.15 * amount)).round() as u8;
            }
        }
    }

    fn into_frame(self) -> SimurghFrame {
        let side = FRAME_WIDTH_PIXELS as usize;
        let mut rgba_pixels = vec![[0_u8; 4]; side * side];
        for (index, pixel) in self.pixels.iter().enumerate() {
            if let Some(color) = pixel {
                let (cell_x, cell_y) = (index % SIDE, index / SIDE);
                for block_y in 0..UPSCALE {
                    for block_x in 0..UPSCALE {
                        rgba_pixels
                            [(cell_y * UPSCALE + block_y) * side + cell_x * UPSCALE + block_x] =
                            *color;
                    }
                }
            }
        }
        SimurghFrame { rgba_pixels }
    }
}

// ---------------------------------------------------------------------------
// Parts

#[derive(Clone, Copy, PartialEq)]
enum WingSide {
    Far,
    Near,
}

fn draw_wing(
    canvas: &mut Canvas,
    skeleton: &Skeleton,
    anatomy: &Anatomy,
    flight_pose: &FlightPose,
    coat: &Coat,
    side: WingSide,
    shine_points: &mut Vec<Vec2>,
) {
    let far = side == WingSide::Far;
    let spread = flight_pose.wing_spread.clamp(0.0, 1.0);
    // The far wing is held a little lower and its farthest tips are hidden by the body.
    let lift = if far {
        flight_pose.wing_lift * 0.8 - 0.3
    } else {
        flight_pose.wing_lift
    };
    let folded_angle = if flight_pose.perched { 2.2 } else { 2.6 };
    let flight_angle = PI + 1.08 * lift;
    let center_angle = folded_angle + (flight_angle - folded_angle) * spread;
    let fan = 0.2 + 1.0 * spread;
    let length_factor = 0.62 + 0.38 * spread;
    let plumage = &coat.palette.plumage;
    let primary_ramp: Ramp = if far {
        [plumage[0], plumage[0], plumage[1], plumage[2]]
    } else {
        [plumage[0], plumage[1], plumage[2], plumage[3]]
    };
    let covert_ramp: Ramp = [plumage[1], plumage[2], plumage[3], plumage[4]];
    let farr = coat.marks.farr_strength;
    let farr_ramp = coat.palette.farr;
    let count = anatomy.wing_count;
    let root = if far {
        add(skeleton.shoulder, scale_by(skeleton.up, 1.5))
    } else {
        skeleton.shoulder
    };
    for (layer_index, (feather_count, layer_length, ramp)) in
        [(count, 1.0, primary_ramp), (count - 1, 0.55, covert_ramp)]
            .into_iter()
            .enumerate()
    {
        for index in 0..feather_count {
            let position = if layer_index == 0 {
                index as f64 / (count - 1) as f64
            } else {
                (index as f64 + 0.5) / (count - 1) as f64
            };
            let angle = center_angle + (position - 0.5) * fan;
            let bump = (PI * (position * 0.8 + 0.1)).sin();
            let length = anatomy.wing_length * length_factor * layer_length * (0.74 + 0.26 * bump);
            let mut path = vec![root];
            let mut spot = root;
            const SEGMENTS: usize = 10;
            for segment in 1..=SEGMENTS {
                let along = segment as f64 / SEGMENTS as f64;
                let heading = angle + flight_pose.wing_bend_radians * along * along;
                spot = add(spot, polar(heading, length / SEGMENTS as f64));
                path.push(spot);
            }
            if layer_index == 0 {
                shine_points.push(spot);
            }
            let widest = (5.6 + 1.4 * layer_index as f64) * anatomy.scale;
            canvas.stroke(
                &path,
                |along| widest * (PI * (along * 0.8 + 0.05)).sin().max(0.0).powf(0.7),
                |sample| {
                    let tone = ribbon_tone(sample);
                    if farr > 0.0
                        && layer_index == 0
                        && sample.along > 0.72
                        && bayer(sample.cell) < farr * ((sample.along - 0.72) / 0.28).min(1.0) * 0.9
                    {
                        Some(pick(&farr_ramp, tone))
                    } else {
                        Some(pick(&ramp, tone))
                    }
                },
            );
        }
    }
}

fn draw_tail(
    canvas: &mut Canvas,
    skeleton: &Skeleton,
    anatomy: &Anatomy,
    flight_pose: &FlightPose,
    coat: &Coat,
    seed: u64,
    shine_points: &mut Vec<Vec2>,
) {
    let count = anatomy.tail_count;
    let inks = coat.assign_tail_inks(count, seed);
    let perched = flight_pose.perched;
    let base_angle = skeleton.axis_angle + PI - if perched { 0.12 } else { 0.38 };
    let fan = if perched { 0.55 } else { 0.95 };
    let gravity = if perched { 9.0 } else { 14.0 } * anatomy.scale;
    let wave_size = (1.2 + 9.0 * flight_pose.tail_strength) * anatomy.scale;
    let farr = coat.marks.farr_strength;
    let farr_ramp = coat.palette.farr;
    let eye_ramp = farr_ramp;
    let mut state = seed ^ 0xA5A5_5A5A_1234_8765;
    let mut order: Vec<usize> = (0..count).collect();
    let middle = (count as f64 - 1.0) / 2.0;
    order.sort_by(|&left, &right| {
        (right as f64 - middle)
            .abs()
            .total_cmp(&(left as f64 - middle).abs())
    });
    for index in order {
        let position = if count == 1 {
            0.5
        } else {
            index as f64 / (count - 1) as f64
        };
        let jitter = (splitmix64(&mut state) % 1000) as f64 / 1000.0 - 0.5;
        let angle = base_angle + (position - 0.5) * fan + jitter * 0.06;
        let centrality = 1.0 - (2.0 * position - 1.0).abs().powf(1.5);
        let length =
            anatomy.tail_length * (0.78 + 0.22 * centrality) * if perched { 0.68 } else { 1.0 };
        let direction = polar(angle, 1.0);
        let sideways = [-direction[1], direction[0]];
        const SEGMENTS: usize = 18;
        let mut path = Vec::with_capacity(SEGMENTS + 1);
        for segment in 0..=SEGMENTS {
            let along = segment as f64 / SEGMENTS as f64;
            let wave = wave_size
                * along.powf(1.4)
                * (TAU * 0.9 * along - flight_pose.tail_phase_radians + index as f64 * 0.55).sin();
            let spot = add(
                add(skeleton.tail_root, scale_by(direction, length * along)),
                add(scale_by(sideways, wave), [0.0, gravity * along * along]),
            );
            path.push(spot);
        }
        shine_points.push(*path.last().expect("path has points"));
        let ramp = inks[index];
        let widest = (5.0 + 1.2 * centrality) * anatomy.scale.max(0.8);
        let eyes = anatomy.tail_eyes;
        canvas.stroke(
            &path,
            |along| {
                let ribbon = 0.45 + 0.55 * (PI * along * 0.9).sin().max(0.0).powf(0.6);
                let bulb = if eyes {
                    1.1 * (-((along - 0.86) / 0.09).powi(2)).exp()
                } else {
                    0.0
                };
                widest * (ribbon + bulb) * (1.0 - along.powi(8)).max(0.0).sqrt()
            },
            |sample| {
                let tone = ribbon_tone(sample) * 1.12;
                if eyes && sample.along > 0.72 {
                    let ring = ((sample.along - 0.86).abs() / 0.12).hypot(sample.across * 0.95);
                    if ring < 0.34 {
                        return Some([20, 28, 78, 255]);
                    } else if ring < 0.62 {
                        return Some(eye_ramp[1]);
                    } else if ring < 0.8 {
                        return Some(eye_ramp[2]);
                    }
                }
                if farr > 0.0
                    && sample.across.abs() > 0.8
                    && sample.along > 0.55
                    && bayer(sample.cell) < farr * 0.8
                {
                    Some(pick(&farr_ramp, tone))
                } else {
                    Some(pick(&ramp, tone))
                }
            },
        );
    }
}

fn draw_legs(
    canvas: &mut Canvas,
    skeleton: &Skeleton,
    anatomy: &Anatomy,
    flight_pose: &FlightPose,
    coat: &Coat,
) {
    let leg_ramp: Ramp = [
        coat.palette.beak[0],
        coat.palette.beak[1],
        coat.palette.beak[1],
        coat.palette.beak[2],
    ];
    let scale = anatomy.scale;
    for lean in [-3.0, 3.0] {
        let hip = add(
            skeleton.center,
            add(
                scale_by(skeleton.axis, lean * scale),
                scale_by(skeleton.up, -anatomy.body_half_height * 0.7),
            ),
        );
        let foot = if flight_pose.perched {
            [hip[0] + lean * 0.3, BRANCH_Y - 1.5]
        } else {
            add(hip, [-5.0 * scale + lean * 0.4, 4.5 * scale])
        };
        canvas.stroke(
            &[hip, foot],
            |_| 2.0,
            |sample| Some(pick(&leg_ramp, ribbon_tone(sample))),
        );
    }
}

fn draw_halo(
    canvas: &mut Canvas,
    skeleton: &Skeleton,
    anatomy: &Anatomy,
    coat: &Coat,
    seed: u64,
    flight_pose: &FlightPose,
) {
    let farr = coat.marks.farr_strength;
    if farr <= 0.0 || flight_pose.head_tuck > 0.5 {
        return;
    }
    const DOTS: usize = 22;
    let lit = (farr * DOTS as f64).ceil() as usize;
    let mut order: Vec<usize> = (0..DOTS).collect();
    let mut state = seed ^ 0x0F0F_F0F0_5555_AAAA;
    for position in (1..DOTS).rev() {
        let chosen = (splitmix64(&mut state) % (position as u64 + 1)) as usize;
        order.swap(position, chosen);
    }
    let radius = anatomy.head_radius + 6.5 * anatomy.scale;
    for &dot_index in order.iter().take(lit) {
        let angle = TAU * dot_index as f64 / DOTS as f64;
        let spot = add(skeleton.head, polar(angle + 0.2, radius));
        let shade = if (dot_index + (flight_pose.sparkle_phase_radians * 2.0 / TAU) as usize)
            .is_multiple_of(2)
        {
            coat.palette.farr[2]
        } else {
            coat.palette.farr[1]
        };
        let (cell_x, cell_y) = (spot[0].floor() as i32, spot[1].floor() as i32);
        if canvas.get(cell_x, cell_y).is_none() {
            canvas.put(cell_x, cell_y, shade);
        }
    }
}

fn draw_body(canvas: &mut Canvas, skeleton: &Skeleton, anatomy: &Anatomy, coat: &Coat) {
    let (half_length, half_height) = (anatomy.body_half_length, anatomy.body_half_height);
    let reach = half_length.ceil() as i32 + 1;
    let (center_x, center_y) = (
        skeleton.center[0].floor() as i32,
        skeleton.center[1].floor() as i32,
    );
    for cell_y in center_y - reach..=center_y + reach {
        for cell_x in center_x - reach..=center_x + reach {
            let offset = [
                f64::from(cell_x) + 0.5 - skeleton.center[0],
                f64::from(cell_y) + 0.5 - skeleton.center[1],
            ];
            let along = dot(offset, skeleton.axis) / half_length;
            let height = dot(offset, skeleton.up) / half_height;
            if along * along + height * height <= 1.0 {
                let facing = normalized(add(
                    scale_by(skeleton.axis, along / half_length),
                    scale_by(skeleton.up, height / half_height),
                ));
                let depth = (1.0 - along * along - height * height).max(0.0).sqrt();
                let tone = 0.5 + 0.5 * (0.8 * dot(facing, TOWARD_LIGHT) + 0.5 * depth - 0.2);
                canvas.put(cell_x, cell_y, pick(&coat.palette.plumage, tone));
            }
        }
    }
}

fn draw_neck_and_head(
    canvas: &mut Canvas,
    skeleton: &Skeleton,
    anatomy: &Anatomy,
    flight_pose: &FlightPose,
    coat: &Coat,
) {
    let plumage = &coat.palette.plumage;
    let ramp: Ramp = [plumage[0], plumage[1], plumage[2], plumage[3]];
    let scale = anatomy.scale;
    let neck_path: Vec<Vec2> = (0..=16)
        .map(|index| {
            let along = f64::from(index) / 16.0;
            quadratic_point(
                skeleton.neck_start,
                skeleton.neck_control,
                skeleton.head,
                along,
            )
        })
        .collect();
    let thick = 7.2 * scale;
    let thin = 5.0 * scale * (1.0 + 0.3 * anatomy.youth);
    canvas.stroke(
        &neck_path,
        |along| thick + (thin - thick) * along,
        |sample| Some(pick(&ramp, ribbon_tone(sample))),
    );
    // A tucked head sits against the back and shows only its crown, so it is drawn
    // under the wing's covert feathers.
    canvas.disc(skeleton.head, anatomy.head_radius, plumage);

    // Crest.
    let crest_ramp: Ramp = [
        coat.palette.crest[0],
        coat.palette.crest[0],
        coat.palette.crest[1],
        coat.palette.crest[2],
    ];
    let beak_angle = skeleton.beak_angle;
    let crest_root = add(
        skeleton.head,
        add(
            polar(beak_angle + PI, anatomy.head_radius * 0.2),
            polar(beak_angle - PI / 2.0, anatomy.head_radius * 0.75),
        ),
    );
    for index in 0..anatomy.crest_count {
        let position = if anatomy.crest_count == 1 {
            0.5
        } else {
            index as f64 / (anatomy.crest_count - 1) as f64
        };
        let angle = beak_angle + PI + 0.15 - 0.95
            + position * 0.95
            + 0.12 * (flight_pose.tail_phase_radians + index as f64).sin();
        let length = anatomy.crest_length * (0.8 + 0.2 * (1.0 - (2.0 * position - 1.0).abs()));
        let mut spot = crest_root;
        let mut path = vec![spot];
        for segment in 1..=6 {
            let along = segment as f64 / 6.0;
            spot = add(spot, polar(angle + 0.5 * along, length / 6.0));
            spot[1] += 0.9 * along * anatomy.scale;
            path.push(spot);
        }
        canvas.stroke(
            &path,
            |along| 2.4 * scale.max(0.7) * (1.0 - along * 0.8),
            |sample| Some(pick(&crest_ramp, ribbon_tone(sample))),
        );
    }

    // Beak.
    let beak_base = add(skeleton.head, polar(beak_angle, anatomy.head_radius * 0.75));
    let beak_length = (4.0 + 3.0 * (1.0 - anatomy.youth * 0.5)) * scale.max(0.75);
    let beak_ramp: Ramp = [
        coat.palette.beak[0],
        coat.palette.beak[1],
        coat.palette.beak[1],
        coat.palette.beak[2],
    ];
    let beak_path = [
        beak_base,
        add(beak_base, polar(beak_angle, beak_length * 0.6)),
        add(
            add(beak_base, polar(beak_angle, beak_length)),
            polar(beak_angle + PI / 2.0, beak_length * 0.18),
        ),
    ];
    canvas.stroke(
        &beak_path,
        |along| 3.2 * scale.max(0.75) * (1.0 - along * 0.9),
        |sample| Some(pick(&beak_ramp, ribbon_tone(sample))),
    );

    // Eye.
    let eye = add(
        add(skeleton.head, polar(beak_angle, anatomy.head_radius * 0.3)),
        polar(beak_angle - PI / 2.0, anatomy.head_radius * 0.22),
    );
    let (eye_x, eye_y) = (eye[0].floor() as i32, eye[1].floor() as i32);
    if flight_pose.eye_open < 0.5 {
        canvas.put(eye_x, eye_y, outline());
        canvas.put(eye_x - 1, eye_y, outline());
    } else {
        canvas.put(eye_x, eye_y, outline());
        canvas.put(eye_x - 1, eye_y - 1, [255, 255, 255, 255]);
        if anatomy.youth > 0.3 {
            canvas.put(eye_x + 1, eye_y, outline());
            canvas.put(eye_x, eye_y + 1, outline());
            canvas.put(eye_x + 1, eye_y + 1, outline());
        }
    }
}

fn draw_branch(canvas: &mut Canvas, seed: u64) {
    let [start_x, end_x] = BRANCH_X_RANGE;
    let path: Vec<Vec2> = (0..=30)
        .map(|index| {
            let along = f64::from(index) / 30.0;
            let x = start_x + (end_x - start_x) * along;
            [x, branch_y(x)]
        })
        .collect();
    canvas.stroke(
        &path,
        |along| 4.6 - 2.0 * along,
        |sample| Some(pick(&BRANCH_RAMP, ribbon_tone(sample))),
    );
    // Leaves along the branch, placed by the seed.
    let mut state = seed ^ 0x1357_9BDF_2468_ACE0;
    for index in 0..9 {
        let x = start_x + 8.0 + (end_x - start_x - 16.0) * (f64::from(index) + 0.5) / 9.0;
        let jitter = (splitmix64(&mut state) % 1000) as f64 / 1000.0;
        let upward = index % 2 == 0;
        let angle = if upward {
            -0.9 - jitter * 0.5
        } else {
            0.8 + jitter * 0.5
        };
        let root = [x, branch_y(x)];
        let tip = add(root, polar(angle, 6.0 + 3.0 * jitter));
        canvas.stroke(
            &[root, tip],
            |along| 3.4 * (PI * along).sin().max(0.15),
            |sample| Some(pick(&LEAF_RAMP, ribbon_tone(sample))),
        );
    }
    for pod_x in pod_positions() {
        let hang = [pod_x, branch_y(pod_x) + 5.5];
        canvas.stroke(
            &[[pod_x, branch_y(pod_x) + 1.0], hang],
            |_| 1.0,
            |_| Some(BRANCH_RAMP[1]),
        );
        canvas.disc(add(hang, [0.0, 1.5]), 2.2, &seed_pod_ramp());
    }
}

fn draw_seeds(canvas: &mut Canvas, flight_pose: &FlightPose, coat: &Coat, seed: u64) {
    let Some(phase) = flight_pose.seed_phase else {
        return;
    };
    let _ = seed;
    for (index, pod_x) in pod_positions().into_iter().enumerate() {
        let progress = (phase + index as f64 / 4.0).fract();
        if !(0.08..0.92).contains(&progress) {
            continue;
        }
        let drop = (progress - 0.08) / 0.84;
        let x = pod_x + 1.6 * (TAU * 2.0 * drop + index as f64).sin();
        let y = branch_y(pod_x) + 9.0 + 34.0 * drop * drop.sqrt();
        let ink = coat.seed_ink(index);
        let (cell_x, cell_y) = (x.floor() as i32, y.floor() as i32);
        for (step_x, step_y) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            canvas.put(cell_x + step_x, cell_y + step_y, ink[1]);
        }
        canvas.put(cell_x, cell_y, ink[3]);
    }
}

fn draw_sparkles(
    canvas: &mut Canvas,
    coat: &Coat,
    shine_points: &[Vec2],
    seed: u64,
    flight_pose: &FlightPose,
) {
    let strength = coat.marks.glimmer_strength;
    if strength <= 0.0 || shine_points.is_empty() {
        return;
    }
    let count = (strength * 9.0).ceil() as usize;
    for index in 0..count {
        let anchor = shine_points[(index * 7 + seed as usize % 5) % shine_points.len()];
        let [x, y] = add(
            anchor,
            [(index % 3) as f64 - 1.0, -1.0 - (index % 2) as f64],
        );
        let (cell_x, cell_y) = (x.floor() as i32, y.floor() as i32);
        let twinkle = 0.5 + 0.5 * (flight_pose.sparkle_phase_radians + index as f64 * 1.7).sin();
        let white = [255, 255, 255, 255];
        let pale = [254, 243, 176, 255];
        canvas.put(cell_x, cell_y, white);
        if twinkle > 0.35 {
            for (step_x, step_y) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                canvas.put(cell_x + step_x, cell_y + step_y, pale);
            }
        }
        if twinkle > 0.75 {
            for (step_x, step_y) in [(2, 0), (-2, 0), (0, 2), (0, -2)] {
                canvas.put(cell_x + step_x, cell_y + step_y, white);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Geometry and shading helpers

fn branch_y(x: f64) -> f64 {
    BRANCH_Y + 2.0 * ((x - BRANCH_X_RANGE[0]) / 38.0).sin() - (x - BRANCH_X_RANGE[0]) * 0.02
}

fn pod_positions() -> [f64; 4] {
    [16.0, 32.0, 52.0, 112.0]
}

fn seed_pod_ramp() -> [Rgba; 4] {
    [
        [115, 62, 57, 255],
        [215, 118, 67, 255],
        [254, 174, 52, 255],
        [254, 231, 97, 255],
    ]
}

/// Tone across a ribbon: bright on the spine, dark at the edges, lit from the upper
/// left, a little brighter toward the tip.
fn ribbon_tone(sample: &Sample) -> f64 {
    let edge = sample.across.abs();
    let side = if sample.across >= 0.0 { 1.0 } else { -1.0 };
    let facing = dot(sample.normal, TOWARD_LIGHT) * side;
    let body = 1.0 - 0.9 * edge.powf(1.7);
    0.65 * body + 0.35 * (0.5 + 0.5 * facing * edge) + 0.08 * sample.along
}

/// The step of a ramp for a tone from zero through one.
fn pick(ramp: &[Rgba], tone: f64) -> Rgba {
    let index = (tone.clamp(0.0, 0.999) * ramp.len() as f64) as usize;
    ramp[index]
}

fn bayer(cell: [i32; 2]) -> f64 {
    BAYER[cell[1].rem_euclid(4) as usize][cell[0].rem_euclid(4) as usize] / 16.0
}

fn polar(angle: f64, length: f64) -> Vec2 {
    [angle.cos() * length, angle.sin() * length]
}

fn add(left: Vec2, right: Vec2) -> Vec2 {
    [left[0] + right[0], left[1] + right[1]]
}

fn scale_by(vector: Vec2, factor: f64) -> Vec2 {
    [vector[0] * factor, vector[1] * factor]
}

fn dot(left: Vec2, right: Vec2) -> f64 {
    left[0] * right[0] + left[1] * right[1]
}

fn distance(left: Vec2, right: Vec2) -> f64 {
    (left[0] - right[0]).hypot(left[1] - right[1])
}

fn normalized(vector: Vec2) -> Vec2 {
    let length = vector[0].hypot(vector[1]);
    if length == 0.0 {
        [0.0, 0.0]
    } else {
        scale_by(vector, 1.0 / length)
    }
}

fn lerp_point(from: Vec2, to: Vec2, amount: f64) -> Vec2 {
    add(scale_by(from, 1.0 - amount), scale_by(to, amount))
}

fn quadratic_point(start: Vec2, control: Vec2, end: Vec2, along: f64) -> Vec2 {
    let near = lerp_point(start, control, along);
    let far = lerp_point(control, end, along);
    lerp_point(near, far, along)
}

/// The point and unit direction a given distance along a path.
fn sample_path(path: &[Vec2], distances: &[f64], traveled: f64) -> (Vec2, Vec2) {
    let segment = distances
        .windows(2)
        .position(|pair| traveled <= pair[1])
        .unwrap_or(path.len() - 2);
    let (from, to) = (path[segment], path[segment + 1]);
    let span = (distances[segment + 1] - distances[segment]).max(f64::EPSILON);
    let amount = ((traveled - distances[segment]) / span).clamp(0.0, 1.0);
    let tangent = normalized([to[0] - from[0], to[1] - from[1]]);
    (lerp_point(from, to, amount), tangent)
}
