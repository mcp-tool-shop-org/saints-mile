# saints-mile: how it works

Mapped at 2026-10-01 from commit f161838 by Atlas 1.24.0.

## What this is

6 parts, mostly Rust (109 files), JavaScript (3), CSS (2), TypeScript (2), Astro (1), PowerShell (1) and shell (1). Work enters through 4 doors; CI and Release Binaries each reach 2 parts, and CI is followed because a pull request goes through it. It publishes to crates.io. It deploys a site to GitHub Pages. People run saints-mile.

## What changed since 2026-09-30 (18fa475)

- CI's pull request trigger no longer names `.github/workflows/**`, `Cargo.lock`, `Cargo.toml`, `atlas/**`, `site/astro.config.mjs`, `site/package-lock.json`, `site/package.json` and `src/**`.
- 1 file changed content, across 1 part.

## What comes in

1. **CI.** On a pull request to main; on a push to main touching 8 paths; or by hand. Runs src/combat/convoy.rs, src/combat/crowd.rs, src/combat/engine.rs and 43 more; checks src/lib.rs and src/main.rs.
2. **Release Binaries.** When a release is published; or by hand. Runs msix/gen-assets.mjs; builds src/main.rs; checks src/lib.rs.
3. **Deploy site to GitHub Pages.** On a push to main touching 2 paths; or by hand. Runs site/astro.config.mjs and site/src/.
4. **saints-mile** (a command people run). Runs src/main.rs.

## What happens through CI

1. The workflow runs 12 files in src and 34 files in tests; it checks src/lib.rs and src/main.rs in src.

## Who reads the results

CI writes nothing this map can see.

## The other doors

**Release Binaries** runs msix/gen-assets.mjs, checks src/lib.rs, writes to msix/Assets/, builds src/main.rs into an MSIX package and binaries for darwin-arm64, linux-x64 and win-x64, and uploads them to the release, and publishes to crates.io on a release event.

**Deploy site to GitHub Pages** runs site/astro.config.mjs and site/src/, and deploys the site.

**saints-mile** (a command people run) runs src/main.rs.

## What breaks what

- **src** is imported only from tests, by 1 part (tests), and sits on the path of 3 doors.
- **msix/Assets/** is written by msix and read by msix; a hand edit reaches every reader.

## What tends to change together

No two source files changed together often enough to name.

Window: 180 days; a pair counts from 3 shared commits, since the window holds fewer than 30 qualifying commits.

## What no test touches

- **msix** is imported by no test.

verify.sh runs in no workflow.

## Written but never read

Every written place has a reader.

## Helpers that look duplicated

No two parts export a helper that looks alike.

## Generated, never hand-edited

- **msix/Assets/** is written by msix/gen-assets.mjs and msix/resize-logo.mjs.

## Hand-authored

People write .github/, the repository root and site/. Nothing in this repository writes to them.

## Where to start

src/main.rs → src/lib.rs → src/ui/mod.rs → src/state/store.rs → src/ui/screens/title.rs → src/ui/screens/scene.rs → src/ui/screens/standoff.rs

Read those in order to follow one run of saints-mile end to end. This path follows saints-mile (a command people run) from its entry, since CI runs only tests and checks.

## What this map cannot see

- 1 import could not be resolved: `msix/resize-logo.mjs` imports `sharp`, which is not declared.
- 6 writes and 1 read go to a path their caller passes, not to this repository.
- 4 writes go to the directory the command is run in (saves/) or a path their caller passes, not to this repository.
- 2 reads go to the directory the command is run in (saves/), not to this repository.
- Statistics confidence is low: fewer than 30 qualifying commits in the window, and fewer than 25 source files reach 10 revisions.

Regenerate with `npx --yes @dogfood-lab/atlas map`.
