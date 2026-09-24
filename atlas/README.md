# saints-mile: how it works

Mapped at 2026-09-24 from commit 49f5266.

## What this is

6 parts, mostly Rust (109 files). Work enters through 4 doors; CI and Release Binaries each reach 2 parts, and CI is followed because a pull request goes through it. It publishes to crates.io. People run saints-mile.

## What changed since the last map

This is the first map.

## What comes in

1. **CI.** On a pull request touching 8 paths; on a push to main touching 8 paths; or by hand. Runs src/combat/convoy.rs, src/combat/crowd.rs, src/combat/engine.rs and 43 more; checks src/lib.rs and src/main.rs.
2. **Release Binaries.** When a release is published; or by hand. Runs msix/gen-assets.mjs; checks src/lib.rs and src/main.rs.
3. **Deploy site to GitHub Pages.** On a push to main touching 2 paths; or by hand. Runs site/astro.config.mjs and site/src/.
4. **saints-mile** (a command people run). Runs src/main.rs.

## What happens through CI

1. The workflow runs 12 files in src and 34 files in tests; it checks src/lib.rs and src/main.rs in src.

## Who reads the results

CI writes nothing this map can see.

## The other doors

**Release Binaries** runs msix/gen-assets.mjs, checks src/lib.rs and src/main.rs, writes to msix/Assets/, publishes to crates.io on a release event, and creates a GitHub release on a release event.

**Deploy site to GitHub Pages** runs site/astro.config.mjs and site/src/, and deploys the site.

**saints-mile** (a command people run) runs src/main.rs.

## What breaks what

- **src** is imported only from tests, by 1 part (tests), and sits on the path of 3 doors.
- **msix/Assets/** is written by msix and read by msix; a hand edit reaches every reader.

## What tends to change together

- **src/combat/engine.rs** and **src/combat/types.rs** changed together in 4 of 8 commits, inside the src part.
- **src/combat/engine.rs** and **src/ui/mod.rs** changed together in 4 of 8 commits, inside the src part.

Confidence is low: fewer than 20 source files reach 10 revisions in the window.

Window: 180 days; a pair counts from 3 shared commits, since 2 source files reach 10 revisions; the floor rises to 10 when 25 do.

## What no test touches

- **msix** is imported by no test.

## Written but never read

Every written place has a reader.

## Helpers that look duplicated

No two parts export a helper that looks alike.

## Generated, never hand-edited

- **msix/Assets/** is written by msix/gen-assets.mjs and msix/resize-logo.mjs.

## Hand-authored

People write .github/, the repository root and site/; 4 writes with paths built at run time may land here.

## Where to start

src/main.rs

Read those in order to follow one run of saints-mile end to end. This path follows saints-mile (a command people run) from its entry, since CI runs only tests.

## What this map cannot see

- 1 import site could not be resolved.
- 4 writes use paths built at run time and are not named here.
- 6 writes and 3 reads go to the directory the command is run in, the home directory, a temporary directory or a path its caller passes, not to this repository.
- Statistics confidence is low: fewer than 20 source files reach 10 revisions in the window.

Regenerate with `npx --yes @dogfood-lab/atlas map`.
