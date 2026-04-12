# Dependency Migration Proposal

This document captures the current dependency state in `encrusted` and the safest next migration steps based on the latest sweep.

## Current Versions From The Repo

JavaScript dependencies are declared in `package.json`:

- `react`: `^17.0.2`
- `react-dom`: `^17.0.2`
- `react-router-dom`: `^5.2.0`
- `react-redux`: `^7.2.3`
- `redux`: `^4.0.5`
- `electron`: `^12.0.2`
- `webpack`: `^5.30.0`
- `webpack-cli`: `^4.6.0`
- `webpack-dev-server`: `^3.11.2`
- `@babel/core`: `^7.13.14`
- `@babel/preset-react`: `^7.13.13`
- `copy-webpack-plugin`: `^8.1.1`

Rust dependencies are declared in `Cargo.toml`:

- `base64`: `0.10.1`
- `rand`: `0.4.2`
- `serde`: `1.0.88`
- `serde_derive`: `1.0.88`
- `serde_json`: `1.0.38`
- `bitflags`: `1.2.1`
- `unicode-segmentation`: `1.7.1`
- `clap`: `2.32.0`
- `regex`: `1.1.0`
- `crossterm`: `0.19.0`

Resolved lockfiles currently present in the repo:

- `package-lock.json`
- `Cargo.lock`

## Recommendation

Do not do a broad dependency upgrade yet.

The safest path is:

1. Restore deterministic JavaScript installs.
2. Re-run the dependency sweep once registry resolution is reliable.
3. Only then consider small build-tool upgrades that stay within the current app architecture.

## Babel Migration Chosen

The repo has now been updated to target the newest stable Babel 7 React preset currently published:

- `@babel/preset-react`: `^7.27.1`
- `@babel/core`: `^7.28.4`

This is a narrow manifest change only. The lockfile was not refreshed as part of this migration because the install path still needs to be verified in a healthy npm environment.

## react-split-pane Status

`react-split-pane` is already on the latest stable release line used by npm:

- current repo version: `^0.1.92`
- npm `latest` dist-tag: `0.1.92`
- npm `next` dist-tag: `2.0.3`

No package change was applied for `react-split-pane` because the repo is already on the stable dist-tag.

The `2.0.3` line was not adopted because it is published on `next`, not `latest`, and its package metadata targets an older React baseline:

- `react: ^16.2.0`
- `react-dom: ^16.2.0`

That means moving to `2.0.3` would not be a clean “latest stable” migration for this React 17 app, and it would add unnecessary compatibility risk without a clear benefit.

## Why No Safe Upgrade Set Was Proposed Yet

The sweep found that the dependency ecosystem visible from this environment does not line up cleanly with the versions pinned in the repo:

- `npm outdated` failed because `@babel/preset-react@^7.13.13` did not resolve from the reachable registry metadata.
- The checked-in `package-lock.json` still pins Babel packages in the `7.13.x` range.
- `npm ls --depth=0` showed multiple missing top-level packages in `node_modules`, so the local install is not a trustworthy baseline.
- Rust latest-version verification could not be completed because Cargo was operating offline in this environment.

That means a fresh install or lockfile refresh would be higher risk than usual, and any upgrade proposal based on that state would be speculative.

## Minimal Next Step

The next safe migration is not a framework migration. It is a reproducibility fix:

- Verify the intended npm registry and package source.
- Recreate a consistent install from `package-lock.json`, or regenerate the lockfile in a known-good environment.
- Re-run the dependency sweep after installs are deterministic.

## Breaking-Change Risks To Avoid For Now

These upgrades are possible later, but they are not minimal:

### React Router `5.x -> newer`

Risk:

- Route APIs and surrounding patterns change substantially.
- Existing routing code would likely need refactors rather than a version bump.

Migration impact:

- Update route declarations and navigation patterns.
- Re-test all story/game launch paths and deep links.

### React Redux `7.x -> newer`

Risk:

- Newer versions shift assumptions around supported React/runtime combinations.
- This can cascade into React upgrades and related library changes.

Migration impact:

- Re-verify connected components, hooks usage, and renderer behavior.

### Electron `12.x -> newer`

Risk:

- Main-process and security-related APIs have changed across majors.
- The app already contains deprecated window-opening behavior in `src/electron/electronmain.js`.

Migration impact:

- Replace deprecated `new-window` handling with current APIs.
- Re-test preload, navigation interception, packaging, and desktop-specific behavior.

### webpack-dev-server `3 -> 4+`

Risk:

- Dev-server CLI/config behavior changes.
- The current `npm run dev` script uses `--content-base`, which is tied to the older dev-server model.

Migration impact:

- Update the `dev` script in `package.json`.
- Re-test local browser development flow.

## Practical Plan

If we want the smallest viable migration sequence, do it in this order:

1. Fix install reproducibility.
2. Re-run the sweep.
3. If the environment stabilizes, consider only same-stack build tool updates first.
4. Leave React Router, React Redux, and Electron major upgrades for dedicated follow-up work.

## Outcome

At the moment, the right call is caution:

- No automatic dependency bump was applied.
- No migration-heavy major upgrade is recommended until installs are deterministic.
- The highest-value next action is to repair or verify the JavaScript dependency resolution path.
