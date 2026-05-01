# Deprecated NPM Package Migration Plan

This plan tracks the deprecated npm warnings listed in `DEPRECATED.md` and the
current dependency graph resolved in `package-lock.json`.

## Summary

The listed deprecated packages are not direct dependencies of the app. They are
all transitive packages pulled in through top-level development dependencies:

- `electron`
- `electron-builder`

That means the first migration path should be normal toolchain updates, not
direct replacement of `rimraf`, `inflight`, `glob`, or `boolean`.

## Applied Migration

The first migration pass has been applied:

- `electron` moved from `^41.2.0` to `^41.4.0`.
- `electron-builder` moved from `^26.8.1` to `^26.9.0`.
- `package-lock.json` was refreshed with `npm install`.

Registry verification on April 30, 2026 showed `electron@41.4.0` as the latest
Electron 41 release and `electron-builder@26.9.0` as the latest Electron
Builder 26 release.

## Remaining Deprecated Package Sources

After the update, `npm ls rimraf inflight glob boolean --all` reports these
remaining paths:

| Warning | Current dependency path |
| --- | --- |
| `boolean@3.2.0` | `electron@41.4.0 -> @electron/get@2.0.3 -> global-agent -> boolean` |
| `glob@7.2.3` | `electron-builder@26.9.0 -> app-builder-lib -> @electron/asar@3.4.1 -> glob` |
| `inflight@1.0.6` | `electron-builder@26.9.0 -> app-builder-lib -> @electron/asar@3.4.1 -> glob -> inflight` |
| `rimraf@2.6.3` | `electron-builder@26.9.0 -> app-builder-lib -> electron-builder-squirrel-windows -> electron-winstaller -> temp -> rimraf` |

The previous `glob@10.5.0` warning through
`@electron/rebuild -> node-gyp -> make-fetch-happen -> cacache -> glob` was
removed by the `electron-builder@26.9.0` update.

## Migration Plan

### 1. Refresh Patch Versions First

Run a normal npm update for the two top-level packages responsible for these
warnings:

```sh
npm update electron electron-builder
```

Then regenerate or review `package-lock.json`.

Re-check the deprecated package tree:

```sh
npm ls rimraf inflight glob boolean --all
```

Status: complete. This removed the `glob@10.5.0` warning but left the upstream
`boolean`, `glob@7`, `inflight`, and `rimraf@2` paths listed above.

### 2. Upgrade Electron Within The Current Major

The repo currently uses Electron `^41.4.0`.

Status: complete for Electron 41. `boolean` still remains through
`@electron/get -> global-agent`, so removing it requires either an upstream
Electron toolchain change or a dedicated Electron major-upgrade workstream.

Validate after the update:

```sh
npm run bundle:electron
npm run electron
```

### 3. Upgrade electron-builder Within The Current Major

Move from `^26.8.1` to the latest compatible `26.x` release.

This targets the deprecated packages coming through `@electron/asar`,
`@electron/rebuild`, `electron-winstaller`, `temp`, `rimraf`, and `glob`.

Status: complete for Electron Builder 26. The remaining `glob@7`, `inflight`,
and `rimraf@2` warnings are still inside Electron Builder's transitive packaging
toolchain.

Validate after the update:

```sh
npm run bundle:electron
npm run pack:electron
```

### 4. If glob, inflight, Or rimraf Remain

If `glob@7`, `inflight`, or `rimraf@2` remain after the package refresh, the
likely blocker is the Windows Squirrel packaging path:

```text
electron-builder-squirrel-windows -> electron-winstaller -> temp -> rimraf
```

At that point, decide whether Squirrel.Windows packaging is required.

If Squirrel.Windows is not required, configure Electron Builder to avoid
Squirrel targets and prefer `nsis`, portable, zip, or platform-specific
defaults. Note that `electron-builder-squirrel-windows` is still present in the
installed Electron Builder dependency graph even when no explicit Squirrel
target is configured, so this may document the packaging intent more than it
changes `npm ls` output.

If Squirrel.Windows is required, keep the warning documented until upstream
replaces the deprecated dependency chain.

### 5. Avoid npm overrides As The First Fix

Do not start by forcing transitive versions with `package.json#overrides`.

These packages are toolchain internals, and some version jumps are not safely
API-compatible. In particular, forcing `rimraf@2` to `rimraf@4+` is not a safe
drop-in replacement.

Use overrides only after testing the packaging commands, and only for packages
with compatible APIs.

## Validation Checklist

- [x] `npm install` produces no deprecated warnings, or only documented
      upstream-blocked warnings.
- [x] `npm ls rimraf inflight glob boolean --all` confirms the remaining
      dependency paths.
- [x] `npm run build:debug`
- [x] `npm run bundle:electron`
- [ ] `npm run pack:electron`
      Blocked on Windows symlink privileges while Electron Builder extracts
      `winCodeSign-2.6.0.7z`; the command gets through bundling, native
      dependency install, Electron download, and packaging startup before
      failing with `Cannot create symbolic link`.
- [ ] `npm run electron`
- [ ] `npm run test`
      Blocked in this PowerShell environment after `cargo build --bin
      encrusted` succeeds because `cmd` cannot execute `./tests/runtests.sh`
      and `bash` is not installed.

## Suggested Execution

Split this into two small changes:

1. Update `electron`, `electron-builder`, and `package-lock.json`.
2. If deprecated packages remain, adjust Electron Builder packaging targets or
   document the upstream blocker.
