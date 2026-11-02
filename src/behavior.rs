//! The day's movement, chosen from recent activity by one ordered set of rules.

use crate::development::{RecentActivity, SimurghDevelopment};

/// One movement per run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Behavior {
    /// Sleeps on the branch of the Tree of All Seeds, head tucked, dimmed and blue-tinted.
    Sleep,
    /// Perches on the branch, folded wings, preening and looking about.
    Perch,
    /// Shakes the Tree of All Seeds from its branch, and seeds fall.
    SeedFall,
    /// Strong, fast wingbeats that sweep the tail wide.
    Soar,
    /// Slow, steady wingbeats and a gently flowing tail; the everyday flight.
    Glide,
}

const DEEP_REST_QUIET_DAYS: u32 = 60;
const REST_QUIET_DAYS: u32 = 14;
/// Totals whose crossing earns a week of soaring.
const SOAR_MILESTONES: [u64; 6] = [1_000, 5_000, 10_000, 25_000, 50_000, 100_000];
/// A busy week is compared with at most this many of the latest complete weeks.
const SOAR_COMPARED_WEEKS: usize = 52;
/// A top tenth means little with fewer complete weeks than this.
const SOAR_MIN_COMPLETE_WEEKS: usize = 10;
/// A busy week is matched or beaten by at most one in this many compared weeks.
const SOAR_WEEK_SHARE_DENOMINATOR: usize = 10;
const DAYS_PER_WEEK: usize = 7;

/// Choose the movement for the day after the observation date. A nightly run judges
/// the day that just ended, and the Simurgh shows the result all of the next day. The
/// first matching rule wins:
///
/// | Behavior | When the observation date |
/// | --- | --- |
/// | Sleep | ends DEEP_REST_QUIET_DAYS or more quiet days |
/// | Perch | ends REST_QUIET_DAYS or more quiet days |
/// | Seed fall | is the first day with contributions in its Monday-Sunday week |
/// | Soar | falls within seven days of the total crossing a milestone |
/// | Soar | is a Sunday that finishes a busy week |
/// | Glide | is anything else |
///
/// A finished week is busy when at most one in ten of the latest
/// SOAR_COMPARED_WEEKS complete weeks before it match or beat its total, and at
/// least SOAR_MIN_COMPLETE_WEEKS weeks are compared. So a steady account glides,
/// and the Simurgh soars at most one day a week beyond milestones. This reads no clock
/// and changes no growth, colors, or marks.
pub fn select_behavior(simurgh_development: &SimurghDevelopment) -> Behavior {
    let recent_activity = &simurgh_development.recent_activity;
    if recent_activity.quiet_day_count >= DEEP_REST_QUIET_DAYS {
        Behavior::Sleep
    } else if recent_activity.quiet_day_count >= REST_QUIET_DAYS {
        Behavior::Perch
    } else if is_first_contribution_of_week(recent_activity) {
        Behavior::SeedFall
    } else if has_crossed_milestone(simurgh_development.total_contributions, recent_activity)
        || is_finished_busy_week(recent_activity)
    {
        Behavior::Soar
    } else {
        Behavior::Glide
    }
}

fn is_first_contribution_of_week(recent_activity: &RecentActivity) -> bool {
    matches!(
        recent_activity.observation_week_contributions.split_last(),
        Some((&observed_day, earlier_days))
            if observed_day > 0 && earlier_days.iter().all(|&day_contributions| day_contributions == 0)
    )
}

fn has_crossed_milestone(total_contributions: u64, recent_activity: &RecentActivity) -> bool {
    let earlier_total = total_contributions.saturating_sub(recent_activity.recent_contributions);
    SOAR_MILESTONES
        .iter()
        .any(|&milestone| earlier_total < milestone && milestone <= total_contributions)
}

fn is_finished_busy_week(recent_activity: &RecentActivity) -> bool {
    let finished_week = &recent_activity.observation_week_contributions;
    let complete_weeks = &recent_activity.complete_week_contributions;
    let compared_weeks =
        &complete_weeks[complete_weeks.len().saturating_sub(SOAR_COMPARED_WEEKS)..];
    let week_total: u64 = finished_week.iter().sum();
    let matching_week_count = compared_weeks
        .iter()
        .filter(|&&week_contributions| week_contributions >= week_total)
        .count();
    finished_week.len() == DAYS_PER_WEEK
        && compared_weeks.len() >= SOAR_MIN_COMPLETE_WEEKS
        && matching_week_count * SOAR_WEEK_SHARE_DENOMINATOR <= compared_weeks.len()
}
