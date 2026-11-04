# Pond scenery: lily pads and rocks

Status: proposed, 2026-11-04. Fork branch `pond` of `yalongwastaken/git-koi`.

## Goal

The koi swims alone on an empty canvas. Give it a pond: lily pads on the
surface and rocks on the bottom, whose number and size follow the account's
GitHub activity, the way every other feature of the koi already does.

## What the code allows today

- The pipeline is one way: `collect` writes a versioned JSON snapshot
  (`schema_version` 1 to 3, unknown fields rejected), `koi` derives development
  from it, `behavior` picks the day's movement, `render` draws a 256 by 256
  transparent frame, `output` encodes it. Everything after collection reads no
  clock, so scenery must come from the snapshot.
- GIF frames allow at most 255 opaque colors and no partial transparency.
  A frame stores only the box around its drawn pixels. Static scenery in the
  corners makes that box the whole frame; LZW compression keeps the cost small.
- On a lap the koi's outermost reach is about 116 px from the frame center
  (`calculate_lap_radius`: at most 88 px, plus fins). Hovering behaviors stay
  within roughly 40 px of the center. The corners, beyond about 120 px from
  the center, are never touched by the fish.
- The Action downloads the release binary named by the upstream `Cargo.toml`.
  A fork cannot be used through `@v1`; it is used either through a fork
  release or by building from source in the profile workflow.

## The stat

Jake suggested "issues made/closed the day before". Findings from Jake's
account on 2026-11-04:

- Closing an issue is not a GitHub contribution and has no `closed-by` search
  qualifier. Issues closed in `jrkubic` repositories in the last 60 days: 0.
- Issues opened by `jrkubic` on 2026-11-03: 6. Pull requests opened: 6.
- `contributionsCollection(from, to)` for a one-day window returns
  `totalIssueContributions` and `totalPullRequestContributions` in one query,
  the same source the contribution calendar already comes from.

Recommendation: issues opened give lily pads, pull requests opened give rocks.
Something new floating up versus something settled on the bottom. Both counts
are public and need no extra token scope.

## Approaches

### A. Snapshot version 4 plus a `pond` module (recommended)

`collect` adds one query per day for the seven days ending on the observation
date and records `daily_activity: [{date, issues_opened, pull_requests_opened}]`.
A new `pond` module turns that into a deterministic scene (which objects, what
size, which slot). `render` paints rocks under the koi and pads over it.

- For: follows the project's architecture, so it can go upstream. Reproducible:
  the same snapshot always draws the same pond. The still PNG gets scenery too.
- Against: the widest change. Touches `collect`, `history` (the version 4
  dance), `koi`, `render`, `main` (caption), docs, changelog, fixtures, and the
  library generator.

### B. Command-line counts

`git-koi render ... --lily-pads N --rocks M`, with N and M computed by a shell
step in the workflow using `gh`. Only `render` and `main` change.

- For: small. Any user maps any stat.
- Against: breaks "the snapshot decides everything"; the stat logic lives in
  YAML; unlikely to be accepted upstream; the still and the GIF can disagree
  with the snapshot they came from.

### C. Composite the scenery onto the GIF in the workflow

No Rust change; a Python or ImageMagick step draws onto every frame.

- For: nothing to fork.
- Against: redraws 90 to 120 frames outside the renderer, fights the per-frame
  palette and disposal rules, and gives upstream nothing.

## Design for approach A

### Snapshot

- `schema_version` 4 adds `daily_activity`, a list of the seven days ending on
  `contribution_history.observed_on` (fewer for an account younger than a
  week), each with `date`, `issues_opened`, and `pull_requests_opened`.
- Version 3 snapshots still parse and render; they just have no pond. A version
  4 snapshot without the field fails, matching how version 3 treats colors.
- `collect` asks `contributionsCollection` once per day. Seven small queries,
  well inside the rate limit.

### Pond rules (`pond` module, pure)

- One lily pad for each of the last seven days with at least one issue opened,
  newest day first. One rock for each day with at least one pull request
  opened. So a burst of activity stays in the pond for a week, then drifts off.
  (Alternative if Jake prefers the literal ask: count only the observation day,
  one pad per issue and one rock per pull request, capped at the slot count.)
- Size tier from that day's count: 1 small, 2 to 3 medium, 4 or more large.
- Slots are fixed and deterministic. Lily pads take the eight rim slots, the
  four corners first and then the four edge midpoints, newest at the top right
  and clockwise from there. Rocks take an inner ring of eight slots at about
  105 px from the center, offset 22.5 degrees from the pad slots, so the koi
  glides over them on a lap.
- A large pad only fits a corner; on an edge slot it is drawn medium. The koi
  may pass under the edge of a pad on its widest lap; that is intended.
- The account seed picks each rock's shape variant so two accounts' ponds
  differ. Nothing else is random.

### Drawing (`render`)

- Rocks are painted first, so the koi swims over them. Pads are painted last,
  over the koi. Both get the existing dark `OUTLINE_COLOR` outline.
- Palette, in the project's Endesga-style colors: pads use a three-shade green
  ramp (38,92,66 / 62,137,72 / 99,199,77) with a pale vein; rocks reuse the
  blue-gray `WHITE_RAMP` lows. Under ten new colors per frame.
- A lily pad is an ellipse with a wedge notch toward the center of the pond; a
  rock is a flattened blob with a lit top edge. Sizes: small 5 px, medium 8 px,
  large 11 px radius for pads; rocks two thirds of that.
- Deep rest tints the fish blue and drops pixels out; scenery is exempt.

### Tests, at public boundaries

- `collect`: the fake client answers per-day queries; the snapshot carries
  seven days; a day GitHub omits fails.
- `history`: version 4 round-trips; version 3 still reads; a version 4 document
  without `daily_activity` is rejected.
- `pond`: zero activity draws nothing; one pad per active day; tier thresholds;
  slot order; determinism across calls.
- `render`: with activity, pixels exist outside the koi's bounds; rocks are
  under the fish and pads over it where they overlap; identical inputs give
  identical frames; GIF encoding still succeeds for every behavior.

### Docs

README sentence, getting-started snapshot description, CHANGELOG Unreleased
entry, a pond sample in the library generator.

## Rollout

1. Implement on fork branch `pond`, run the six CI gates, open a pull request
   to `yalongwastaken/git-koi`. CONTRIBUTING asks for one topic per PR, a
   `area: imperative` subject, and documentation in the same change.
2. Until it is merged and released, point Jake's profile workflow
   (`jrkubic/jrkubic`, `.github/workflows/koi.yml`) at the fork: install Rust,
   `cargo install --git https://github.com/jrkubic/git-koi --branch pond --locked`,
   then run the same `collect` and `render` steps the Action runs, with
   `Swatinem/rust-cache` so the nightly build is fast.
3. The website hot-links the profile GIF, so it needs no change.

## Prerequisite

Rust is not installed on this PC (no `cargo`, no `~/.cargo`). Running the
gates locally needs rustup; otherwise the fork's CI is the only check.
