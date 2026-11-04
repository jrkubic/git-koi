//! The pond around the koi: lily pads for the week's issues and rocks for its pull
//! requests, laid out from a development without any pixel geometry.
//!
//! Each day of the [`crate::koi::POND_DAY_COUNT`] days ending on the observation date
//! that opened at least one issue floats one lily pad, and each day that opened at
//! least one pull request settles one rock. The day's count sets the object's size, and
//! the objects take slots in order of age, newest first, so a burst of activity stays
//! in the pond for a week and then drifts off. The account seed picks each slot's
//! shape variant, so two accounts' ponds differ. Where the slots sit, and how large
//! each size is, belongs to [`crate::render`].

use crate::koi::KoiDevelopment;

/// What a pond object is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PondObjectKind {
    /// Floats on the surface, over the koi; one per day with an issue opened.
    LilyPad,
    /// Rests on the bottom, under the koi; one per day with a pull request opened.
    Rock,
}

/// How large a pond object is, from the day's count.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PondObjectSize {
    /// One opened that day.
    Small,
    /// Two or three opened that day.
    Medium,
    /// Four or more opened that day.
    Large,
}

/// One lily pad or rock.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PondObject {
    /// Lily pad or rock.
    pub kind: PondObjectKind,
    /// From the day's count.
    pub size: PondObjectSize,
    /// The object's place among its kind, newest first, below [`SLOT_COUNT`].
    pub slot: usize,
    /// Days between the object's day and the observation date; zero for the day that
    /// just ended.
    pub age_days: u32,
    /// Which of [`VARIANT_COUNT`] shapes the object takes, fixed per account and slot.
    pub variant: u8,
}

/// The day's pond.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Pond {
    /// Newest first; each slot is its index here.
    pub lily_pads: Vec<PondObject>,
    /// Newest first; each slot is its index here.
    pub rocks: Vec<PondObject>,
}

/// Slots for each kind; the pond's week never needs more.
pub const SLOT_COUNT: usize = 8;
/// Shape variants each kind comes in.
pub const VARIANT_COUNT: u8 = 4;
const MEDIUM_COUNT: u32 = 2;
const LARGE_COUNT: u32 = 4;

/// Lay out the pond from the development's daily activity.
///
/// Days are taken newest first. A day with at least one issue opened adds a lily pad
/// and a day with at least one pull request opened adds a rock; a day with both adds
/// one of each. The object's size is small for one, medium for two or three, and
/// large for four or more. Its slot is its index among its kind, so the newest object
/// of each kind takes slot zero. Its variant comes from the account seed and the slot.
/// A development without daily activity gives an empty pond. This reads no clock and
/// changes nothing on the koi.
pub fn lay_out_pond(koi_development: &KoiDevelopment) -> Pond {
    let mut pond = Pond::default();
    for day_activity in koi_development.daily_activity.iter().rev() {
        let age_days = (koi_development.observed_on - day_activity.date)
            .num_days()
            .max(0) as u32;
        for (kind, day_count, objects) in [
            (
                PondObjectKind::LilyPad,
                day_activity.issues_opened,
                &mut pond.lily_pads,
            ),
            (
                PondObjectKind::Rock,
                day_activity.pull_requests_opened,
                &mut pond.rocks,
            ),
        ] {
            if day_count == 0 || objects.len() >= SLOT_COUNT {
                continue;
            }
            let slot = objects.len();
            objects.push(PondObject {
                kind,
                size: size_for_count(day_count),
                slot,
                age_days,
                variant: select_variant(koi_development.identity_seed, kind, slot),
            });
        }
    }
    pond
}

fn size_for_count(day_count: u32) -> PondObjectSize {
    if day_count >= LARGE_COUNT {
        PondObjectSize::Large
    } else if day_count >= MEDIUM_COUNT {
        PondObjectSize::Medium
    } else {
        PondObjectSize::Small
    }
}

/// Two seed bits per slot, from different halves of the seed for the two kinds, so a
/// slot's variant is stable for an account and the kinds vary independently.
fn select_variant(identity_seed: u64, kind: PondObjectKind, slot: usize) -> u8 {
    let kind_offset = match kind {
        PondObjectKind::LilyPad => 0,
        PondObjectKind::Rock => 32,
    };
    let mixed_seed = identity_seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    ((mixed_seed >> (kind_offset + 2 * slot)) % u64::from(VARIANT_COUNT)) as u8
}
