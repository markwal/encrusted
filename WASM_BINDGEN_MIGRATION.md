# Wasm Bindgen Migration And Rust Environment Prep

This document captures the approved plan to migrate `encrusted` away from `wasm-ffi` and the Rust/Cargo environment decisions we should lock down before changing the wasm bridge.

## Summary

- Replace the `wasm-ffi` bridge in `src/js/worker.js` with a `wasm-bindgen`-based interface.
- Modernize the Rust wasm export layer in `src/rust/main.web.rs` to expose a bindgen-friendly `Engine` API instead of raw exported functions and manual allocation helpers.
- Keep the existing worker message protocol stable so the React and Electron app layers do not need to change while the wasm boundary is being migrated.
- Treat Rust/Cargo setup as part of the migration: make the repo's toolchain expectations explicit, verify current wasm support on stable Rust, and document the future `wasm-bindgen` build flow before refactoring runtime code.

## Environment Audit

Audit performed against the local repository and toolchain on 2026-04-20.

### Repo State

- No repo-specific `.cargo/config` or `.cargo/config.toml` is present.
- No repo-managed Rust toolchain file was present before this prep pass.
- Current Cargo wasm output is the raw Rust artifact:
  - debug: `target/wasm32-unknown-unknown/debug/web.wasm`
  - release: `target/wasm32-unknown-unknown/release/web.wasm`
- Current npm/webpack flow copies that raw wasm artifact directly:
  - `package.json` `build:debug`
  - `package.json` `build:release`
  - `webpack.dev.js`
  - `webpack.electron.js`

### Local Tooling Findings

- `rustc --version`: `rustc 1.94.1 (e408947bf 2026-03-25)`
- `cargo --version`: `cargo 1.94.1 (29ea6fb6a 2026-03-24)`
- Active default toolchain: `stable-x86_64-pc-windows-msvc`
- Installed toolchains also include `nightly-x86_64-pc-windows-msvc`
- Installed Rust targets include `wasm32-unknown-unknown`
- Installed `wasm-bindgen` CLI: `0.2.47`

### Validation Results

- `cargo check --bin encrusted`: passes on stable Rust
- `cargo check --lib --target wasm32-unknown-unknown`: passes on stable Rust

Conclusion: the repository does not currently require nightly Rust for its existing terminal or wasm check flows. The README's nightly-only setup guidance is stale and should be treated as historical, not current policy.

## Toolchain Decisions

### Current Decision

- Use stable Rust as the default repo toolchain.
- Require the `wasm32-unknown-unknown` target for web/wasm work.
- Do not require nightly unless a future migration step introduces a concrete nightly-only need.

### Repo Guardrails

- Add `rust-toolchain.toml` with `stable` and `wasm32-unknown-unknown` so a fresh checkout gets the correct channel/target expectations automatically.
- Keep Cargo configuration otherwise minimal; there is no evidence yet that repo-specific linker or target overrides are needed.

### Wasm Bindgen CLI

- The locally installed `wasm-bindgen` CLI is present but old (`0.2.47`).
- When the actual bridge migration begins, install or upgrade `wasm-bindgen-cli` to match the `wasm-bindgen` crate version added to `Cargo.toml`.
- Do not assume the existing CLI is compatible with the future crate version.

## Approved Migration Plan

### Rust wasm boundary

- Add `wasm-bindgen` as a wasm-only dependency in `Cargo.toml`.
- Rewrite `src/rust/main.web.rs` to export a `#[wasm_bindgen] pub struct Engine` that owns the `Zmachine` instance.
- Expose these methods on `Engine`:
  - `new(file: &[u8])`
  - `step() -> bool`
  - `feed(input: &str)`
  - `restore(data: &str)`
  - `load_savestate(data: &str)`
  - `get_updates()`
  - `undo() -> bool`
  - `redo() -> bool`
  - `enable_instruction_logs(enabled: bool)`
  - `get_object_details(obj_num: u16) -> String`
  - `flush_log()`
  - `set_terp_caps(json: &str)`
- Replace raw imported JS functions with `#[wasm_bindgen] extern "C"` imports for:
  - `js_message`
  - `consolelog`
  - `js_error`
- Remove raw ABI helpers that become unnecessary with bindgen:
  - `allocate`
  - `deallocate`
  - manual `CStr` string decoding helpers
  - thread-local `ZVM` wrapper

### JS worker integration

- Replace `wasm-ffi` usage in `src/js/worker.js` with the generated `wasm-bindgen` module.
- Keep one worker-local engine instance and one wasm init path.
- Preserve incoming worker commands:
  - `instantiate`
  - `load`
  - `start`
  - `restart`
  - `input`
  - `restore`
  - `load_savestate`
  - `undo`
  - `redo`
  - `interpreter_header`
  - `enable:instructions`
  - `getDetails`
- Preserve outgoing worker events and payload shapes so Redux and Electron renderer behavior stays stable.
- Replace manual wasm memory writes for story-file loading with direct bindgen byte-array passing.
- Replace the current `.value` and `.free()` handling for `get_object_details()` with direct JS string use.

### Build and packaging

- Remove `wasm-ffi` from `package.json`.
- Change the build flow to:
  1. `cargo build --lib --target wasm32-unknown-unknown`
  2. `wasm-bindgen <raw wasm> --out-dir <generated-dir> --target bundler`
- Use a generated wasm output directory that webpack can consume without mixing raw Cargo artifacts and processed bindgen output.
- Update `build:debug`, `build:release`, and Electron packaging flows to copy or bundle bindgen-generated assets instead of the raw Cargo wasm file.
- Update webpack config only as needed to consume generated bindgen JS and the processed wasm asset.

## Pre-Migration Checklist

- Confirm the repo still builds on stable Rust:
  - `cargo check --bin encrusted`
  - `cargo check --lib --target wasm32-unknown-unknown`
- Confirm the `wasm32-unknown-unknown` target is installed:
  - `rustup target list --installed`
- Confirm the repo-level toolchain declaration matches the intended default channel and target.
- Confirm current scripts and webpack configs still point at the raw `web.wasm` artifact so the bindgen output swap is deliberate.
- Install or upgrade `wasm-bindgen-cli` once the matching crate version is selected.
- Keep terminal behavior out of scope for the migration unless a shared Rust API change makes a terminal adjustment unavoidable.

## Validation Commands

Use the smallest commands that validate the current setup before implementation:

```sh
rustc --version
cargo --version
rustup show
rustup target list --installed
cargo check --bin encrusted
cargo check --lib --target wasm32-unknown-unknown
```

Useful follow-up checks during the actual migration:

```sh
npm run build:debug
npm run bundle:electron
npm run test
```

## Assumptions And Scope

- This prep pass does not implement the `wasm-bindgen` runtime migration yet.
- Terminal support remains unchanged; the migration targets the wasm/web worker boundary.
- The worker protocol is intentionally preserved to limit churn in app code outside the wasm boundary.
- Stable Rust is the default unless a later migration step proves otherwise with a specific technical constraint.
