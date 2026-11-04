# Pond Scenery Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Draw lily pads and rocks around the koi, one lily pad for each of the last seven days with an issue opened and one rock for each day with a pull request opened, sized by that day's count.

**Architecture:** `collect` asks GitHub for each of the seven days' issue and pull request totals and stores them in a version 4 snapshot field `daily_activity`. `koi` validates and carries them on `KoiDevelopment`. A new pure `pond` module turns them into a scene of slots, sizes, and variants. `render` paints rocks under the finished koi and lily pads over it, in frame space, so framing and depth never move them. Design: `docs/plans/2026-11-04-pond-scenery-design.md`.

**Tech Stack:** Rust 2024 edition, serde, chrono, the existing `gif`/`png` encoders. Gates: `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked`, `cargo build --release --locked`, `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps`, `npx prettier@3.9.10 --check .`. Cargo lives at `~/.cargo/bin`.

---

### Task 1: Snapshot version 4 carries `daily_activity`

**Files:**
- Modify: `src/history.rs` (AccountSnapshot, SnapshotDocument, SnapshotVersion)
- Create: `tests/fixtures/account-with-pond.json` (version 4 copy of `account-with-history.json` plus seven days of activity ending 2026-01-06)
- Test: `tests/history.rs`

**Step 1: Failing tests** in `tests/history.rs`:

```rust
#[test]
fn version_four_snapshots_carry_daily_activity_and_round_trip() {
    let account_snapshot: AccountSnapshot = serde_json::from_str(ACCOUNT_WITH_POND_JSON).unwrap();
    let daily_activity = account_snapshot.daily_activity.as_ref().unwrap();
    assert_eq!(daily_activity.len(), 7);
    assert_eq!(daily_activity[6], DayActivity { date: "2026-01-06".parse().unwrap(), issues_opened: 2, pull_requests_opened: 0 });
    let restored: AccountSnapshot = serde_json::from_slice(&serde_json::to_vec(&account_snapshot).unwrap()).unwrap();
    assert_eq!(restored, account_snapshot);
    let written: serde_json::Value = serde_json::to_value(&account_snapshot).unwrap();
    assert_eq!(written["schema_version"], 4);
}

#[test]
fn daily_activity_belongs_to_version_four_only() {
    // v4 without the field, v3 with it, and a day with an unknown field are all rejected.
}
```

**Step 2:** `cargo test --locked --test history` fails: no field `daily_activity`, no `DayActivity`.

**Step 3:** Implement. `DayActivity { date: NaiveDate, issues_opened: u32, pull_requests_opened: u32 }` with `deny_unknown_fields`, Clone/Debug/Eq/PartialEq/Serialize/Deserialize. `AccountSnapshot.daily_activity: Option<Vec<DayActivity>>` doc: "Issues and pull requests opened on each of the last seven observed days; `None` before snapshot version 4." `SnapshotDocument` gets the same optional field with `deserialize_present_activity`. `SnapshotVersion::V4 = 4`. Rules in `TryFrom`: V1..V3 with activity → error "daily_activity is not a field before snapshot version 4"; V4 without → "missing daily_activity in snapshot version 4; collect the snapshot again"; V4 also needs history and colors (extend the existing arms). `From` picks V4 when activity is present. Update the struct doc comment.

**Step 4:** `cargo test --locked --test history` passes. **Step 5:** commit `history: carry a week of issue and pull request counts`.

### Task 2: `koi` validates and carries the week

**Files:**
- Modify: `src/koi.rs` (KoiDevelopment, derive_koi_development, KoiDevelopmentError)
- Test: `tests/koi.rs`

**Step 1: Failing tests:** development from the pond fixture has `daily_activity` of seven entries oldest first; a v3 snapshot gives an empty vec; a day dated after `observed_on` fails `KoiDevelopmentError::DailyActivity(DailyActivityError::AfterObservationDate(date))`; a day more than six days before fails `OutsideWeek(date)`; two entries for one date with different counts fail `ConflictingDate(date)`; identical duplicates merge.

**Step 3:** Add `pub daily_activity: Vec<DayActivity>` to `KoiDevelopment` ("oldest first, within the seven days ending on `observed_on`; empty when the snapshot predates them"). Add `pub enum DailyActivityError { AfterObservationDate(NaiveDate), OutsideWeek(NaiveDate), ConflictingDate(NaiveDate) }` with Display + Error, and `KoiDevelopmentError::DailyActivity(DailyActivityError)` wired into Display/source. `const POND_DAY_COUNT: u64 = 7` public as `pub const POND_DAY_COUNT`. Normalize through a BTreeMap keyed by date.

**Step 5:** commit `koi: carry the week's issues and pull requests`.

### Task 3: `collect` gathers the week

**Files:**
- Modify: `src/collect.rs` (collect_account_snapshot, new `collect_daily_activity`, new `ACTIVITY_QUERY`)
- Test: `tests/collect.rs` (RecordedGitHub gains `activity: fn(NaiveDate) -> Result<Value, CollectionError>`; dispatch on `contributionCalendar` vs `totalIssueContributions`)

**Step 1: Failing tests:** the collected snapshot's `daily_activity` is the seven days 2025-12-27..2026-01-02 with the fake's synthetic counts (`issues = ordinal % 3`, `prs = ordinal % 2`); the activity requests are seven single-day windows `T00:00:00Z`..`T23:59:59Z`; an account created 2026-01-01 gets two days; an account created today gets an empty list; a missing `totalIssueContributions` field fails with UnexpectedResponse naming it.

**Step 3:** `ACTIVITY_QUERY = "query($login: String!, $from: DateTime!, $to: DateTime!) { user(login: $login) { contributionsCollection(from: $from, to: $to) { totalIssueContributions totalPullRequestContributions } } }"`. `collect_daily_activity(github, login, created_on, observed_on)` loops `max(created_on, observed_on - 6)..=observed_on`. Counts read with `read_u64` then `u32::try_from`. Fill `daily_activity: Some(...)`. Update the module and function docs (public data, one request per day).

**Step 5:** commit `collect: request the week's issue and pull request counts`.

### Task 4: the `pond` module

**Files:**
- Create: `src/pond.rs`; register in `src/lib.rs` (module list and pipeline doc step 5)
- Test: `tests/pond.rs`

```rust
//! The pond around the koi: lily pads for issues and rocks for pull requests.
pub enum PondObjectKind { LilyPad, Rock }
pub enum PondObjectSize { Small, Medium, Large }
pub struct PondObject { pub kind: PondObjectKind, pub size: PondObjectSize, pub slot: usize, pub age_days: u32, pub variant: u8 }
pub struct Pond { pub lily_pads: Vec<PondObject>, pub rocks: Vec<PondObject> }
pub const SLOT_COUNT: usize = 8;
pub const VARIANT_COUNT: u8 = 4;
pub fn lay_out_pond(koi_development: &KoiDevelopment) -> Pond
```

Rules (doc comment): walk `daily_activity` newest first; a day with `issues_opened > 0` adds a lily pad, a day with `pull_requests_opened > 0` adds a rock; `slot` is the object's index among its kind (newest 0); size: 1 Small, 2–3 Medium, 4+ Large; `age_days = observed_on - date`; `variant = (identity_seed >> (2 * slot)) % VARIANT_COUNT` (slot-stable per account). Reads no clock.

**Step 1: Failing tests:** empty week → empty pond; a day with issues only → one pad, no rock; counts 1/2/3/4/9 → Small/Medium/Medium/Large/Large; slots count up newest first with ages; variants are stable across calls and differ between seeds 0 and u64::MAX for some slot.

**Step 5:** commit `pond: lay out lily pads and rocks from the week`.

### Task 5: `render` paints the pond

**Files:**
- Modify: `src/render.rs` (`render_pose`: build pond, paint rocks under and pads over; new `paint_pond_layer`, `paint_lily_pad`, `paint_rock`, slot geometry; scenery ramps)
- Test: `tests/render.rs`

Geometry in native pixels from the frame center (128,128):
- Pad slots, newest first: (104,-104), (104,104), (-104,104), (-104,-104), (0,-112), (112,0), (0,112), (-112,0). Pad half widths: Small 5, Medium 8, Large 11; height 0.8 of width; Large on an edge slot (index ≥ 4) draws Medium. Notch: wedge of ±0.38 rad about the variant's angle (variant × 90° + 45°), leaving the center pixel region.
- Rock slots at radius 105, angles (degrees, screen clockwise from right): -67.5, 22.5, 112.5, 202.5, -22.5, 67.5, 157.5, 247.5. Rock half widths: Small 4, Medium 6, Large 8; height 0.65 of width, variant squashes width by (1 − 0.08 × variant) and raises height by the same.
- Ramps: `LILY_RAMP = [[38,92,66],[62,137,72],[99,199,77]]`, `ROCK_RAMP = [[58,68,102],[90,105,136],[139,155,180]]` (opaque). Shading uses `choose_shade` with brightness `0.5 + 0.5·(−0.45·nx − 0.6·ny)` on the normalized offset, so the upper left is lit, plus `OUTLINE_COLOR` through `paint_outline` on the layer.
- Layering in `render_pose`: finish the koi as today (outline, feeding, depth), then `composite_under(rocks_layer)` into transparent pixels only, then `composite_over(pads_layer)`.

**Step 1: Failing tests:**
- `a_quiet_week_leaves_the_frame_as_before`: fixture v3 development renders identical to the same development with an empty `daily_activity` (trivially) and has no pixels beyond 120 px from the center.
- `rocks_lie_under_the_koi`: a development with seven pull-request days: every pixel that differs from the pond-less frame was transparent there; at least 7 distinct rock blobs (count differing pixels > 0).
- `lily_pads_float_over_the_koi`: a `MAX_GROWTH_CONTRIBUTIONS` koi with five medium pads across the cruise loop: some frame has an opaque pond-less pixel that changed, and pads exist in every frame.
- `pond_objects_grow_with_the_day_count`: pad pixel counts strictly increase for 1, 2, 4 issues on a single day.
- `every_behavior_still_encodes_as_a_gif_with_a_full_pond`: for each behavior, `encode_gif` over all frames succeeds.

**Step 5:** commit `render: paint lily pads over the koi and rocks beneath it`.

### Task 6: command caption, docs, changelog, library

**Files:**
- Modify: `src/main.rs` (USAGE text; caption gains ", N lily pads, M rocks" via `describe_pond`), `README.md`, `docs/getting-started.md` (what collect reads; what the pond shows), `CHANGELOG.md` (Unreleased → Added), `examples/generate-library.rs` (a "Pond" section with six samples and `pond-light.png`/`pond-dark.png` overviews), `docs/plans/*` stay out of the upstream PR.
- Test: `tests/render_command.rs` (render the pond fixture; stdout contains "lily pad"; the preview caption too).

**Step 5:** commit `docs: describe the pond`, `main: name the pond in the caption`, `example: add pond samples to the library`.

### Task 7: gates and the profile workflow

1. Run all six gates; fix what they report.
2. `cargo run --locked --example generate-library -- library`; open `library/pond-light.png` and eyeball it.
3. Render Jake's real snapshot: `git-koi collect jrkubic --output snapshot.json && git-koi render snapshot.json --output-dir koi --body asagi`; view `koi/koi.gif`.
4. Push `pond` to the fork; update `jrkubic/jrkubic` `.github/workflows/koi.yml` on a branch to build from the fork (rustup already on ubuntu runners; `Swatinem/rust-cache@v2`; `cargo install --git https://github.com/jrkubic/git-koi --branch pond --locked`; then the same collect/render steps as the Action). Dispatch it and confirm the GIF changes.
