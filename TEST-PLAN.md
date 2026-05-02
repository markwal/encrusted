# Rust Testing And Stream-Mode Plan

## Summary

Add a real Rust test foundation and an explicit terminal `--stream` mode for deterministic stdin/stdout tests. `--stream` will be opt-in, exit cleanly on pipe EOF, and avoid crossterm/status-bar behavior so process-output baselines are stable.

## Key Changes

- Make `cargo test` work on the host by adjusting the Rust library setup:
  - Keep the existing wasm output name/behavior for web builds.
  - Add `rlib` support and `cfg(target_arch = "wasm32")` guards around wasm-only exports/dependencies in `src/rust/main.web.rs`.
  - Re-export core modules needed by tests without changing browser/Electron behavior.
- Update the `UI` trait input contract:
  - Change line input to return `Option<String>` so EOF is distinguishable from an empty command.
  - Change char input similarly if needed for `read_char`.
  - Add a halt flag in `Zmachine`; EOF during read sets it and lets terminal `run()` exit cleanly.
- Add explicit terminal stream mode:
  - New CLI flag: `--stream`.
  - In stream mode, `TerminalUI` never enables raw mode, never emits crossterm cursor/scroll/status updates, never shows `[Hit any key to exit.]`, and writes plain text only.
  - `--stream` exits with code `0` when stdin reaches EOF.
  - Normal terminal behavior remains unchanged unless `--stream` is passed.
- Add Rust process baseline tests:
  - Add dev dependencies such as `assert_cmd`, `predicates`, and optionally `insta` or plain checked-in fixture files.
  - Add integration tests that spawn `target/debug/encrusted --stream --width 60 tests/minizork.z3`, pipe scripted input, and compare normalized stdout to committed baselines.
  - Keep the existing Python `regtest.py` suite; do not replace it.

## Test Plan

- `cargo test --no-run` must pass on the host.
- `cargo test` must run unit tests plus the new process baseline tests.
- Add unit tests with fake UI coverage for:
  - EOF during line input stops the VM without panic.
  - Empty line input is still treated as a real empty command, not EOF.
  - Stream mode suppresses terminal-only status/drop prompts.
- Add one narrow baseline smoke test first, then expand to more scripted transcripts once the format is stable.
- Continue using `npm run test`/`tests/runtests.sh` for existing Z-machine regression coverage.

## Assumptions

- The stream mode is explicit flag-only, per preference: no automatic pipe detection behavior change.
- v4+ games are not guaranteed to produce canonical stream baselines; they can still be run with `--stream`, but baseline coverage should start with v1-v3 style line-input transcripts.
- Generated outputs in `build/` and `target/` stay untouched.
