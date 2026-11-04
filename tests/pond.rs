use std::collections::BTreeSet;

use chrono::Days;
use git_koi::history::{AccountSnapshot, DayActivity};
use git_koi::koi::{KoiDevelopment, derive_koi_development};
use git_koi::pond::{
    Pond, PondObject, PondObjectKind, PondObjectSize, SLOT_COUNT, VARIANT_COUNT, lay_out_pond,
};

#[test]
fn a_quiet_week_leaves_an_empty_pond() {
    assert_eq!(lay_out_pond(&develop_with_week(&[])), Pond::default());
    assert_eq!(
        lay_out_pond(&develop_with_week(&[(0, 0), (0, 0), (0, 0)])),
        Pond::default()
    );
}

#[test]
fn each_active_day_floats_a_lily_pad_for_issues_and_settles_a_rock_for_pull_requests() {
    // Oldest first: six days ago through the observation date.
    let pond = lay_out_pond(&develop_with_week(&[
        (1, 0),
        (0, 0),
        (0, 2),
        (3, 3),
        (0, 0),
        (0, 0),
        (4, 0),
    ]));

    let describe = |objects: &[PondObject]| -> Vec<(usize, u32, PondObjectSize)> {
        objects
            .iter()
            .map(|pond_object| (pond_object.slot, pond_object.age_days, pond_object.size))
            .collect()
    };
    assert_eq!(
        describe(&pond.lily_pads),
        [
            (0, 0, PondObjectSize::Large),
            (1, 3, PondObjectSize::Medium),
            (2, 6, PondObjectSize::Small),
        ]
    );
    assert_eq!(
        describe(&pond.rocks),
        [
            (0, 3, PondObjectSize::Medium),
            (1, 4, PondObjectSize::Medium)
        ]
    );
    assert!(
        pond.lily_pads
            .iter()
            .all(|pond_object| pond_object.kind == PondObjectKind::LilyPad)
    );
    assert!(
        pond.rocks
            .iter()
            .all(|pond_object| pond_object.kind == PondObjectKind::Rock)
    );
}

#[test]
fn size_follows_the_days_count() {
    for (day_count, expected_size) in [
        (1, PondObjectSize::Small),
        (2, PondObjectSize::Medium),
        (3, PondObjectSize::Medium),
        (4, PondObjectSize::Large),
        (90, PondObjectSize::Large),
    ] {
        let pond = lay_out_pond(&develop_with_week(&[(day_count, day_count)]));
        assert_eq!(pond.lily_pads[0].size, expected_size, "{day_count} issues");
        assert_eq!(
            pond.rocks[0].size, expected_size,
            "{day_count} pull requests"
        );
    }
}

#[test]
fn a_full_week_fits_in_the_slots() {
    let pond = lay_out_pond(&develop_with_week(&[(1, 1); 7]));
    assert_eq!(pond.lily_pads.len(), 7);
    assert_eq!(pond.rocks.len(), 7);
    assert!(
        pond.lily_pads
            .iter()
            .chain(&pond.rocks)
            .all(|pond_object| pond_object.slot < SLOT_COUNT)
    );
}

#[test]
fn variants_are_stable_per_account_and_differ_between_accounts() {
    let koi_development = develop_with_week(&[(1, 1); 7]);
    let pond = lay_out_pond(&koi_development);
    assert_eq!(lay_out_pond(&koi_development), pond);
    assert!(
        pond.lily_pads
            .iter()
            .chain(&pond.rocks)
            .all(|pond_object| pond_object.variant < VARIANT_COUNT)
    );

    let variants_for = |identity_seed: u64| -> Vec<u8> {
        let mut other_development = develop_with_week(&[(1, 1); 7]);
        other_development.identity_seed = identity_seed;
        let other_pond = lay_out_pond(&other_development);
        other_pond
            .lily_pads
            .iter()
            .chain(&other_pond.rocks)
            .map(|pond_object| pond_object.variant)
            .collect()
    };
    let distinct_ponds: BTreeSet<Vec<u8>> = (1..=12).map(variants_for).collect();
    assert!(distinct_ponds.len() > 1);
}

/// A development whose week is the given (issues, pull requests) pairs, oldest first,
/// ending on the observation date.
fn develop_with_week(week: &[(u32, u32)]) -> KoiDevelopment {
    let mut account_snapshot: AccountSnapshot =
        serde_json::from_str(include_str!("fixtures/account-with-pond.json")).unwrap();
    let observed_on = account_snapshot
        .contribution_history
        .as_ref()
        .unwrap()
        .observed_on;
    account_snapshot.daily_activity = Some(
        week.iter()
            .rev()
            .enumerate()
            .map(
                |(age_days, &(issues_opened, pull_requests_opened))| DayActivity {
                    date: observed_on - Days::new(age_days as u64),
                    issues_opened,
                    pull_requests_opened,
                },
            )
            .collect(),
    );
    derive_koi_development(&account_snapshot).unwrap()
}
