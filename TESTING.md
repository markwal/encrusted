# Testing Philosophy

Testing in this repo should make interpreter changes safer without making ordinary development heavy. The goal is to cover small Rust behaviors close to the code, then use a smaller number of process-level tests to prove that the terminal interpreter behaves correctly from the outside.

Prefer the smallest test that catches the bug or protects the behavior:

- Use Rust unit tests for pure or nearly-pure function behavior.
- Use fake UI implementations for `Zmachine` tests that need input or output.
- Use process/baseline tests for terminal behavior, piping, CLI flags, and full story transcripts.
- Keep the existing Python regression suite for Z-machine script compatibility checks.

## Rust Unit Tests

Function-level Rust tests should usually live in the same source file as the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_input_before_tokenising() {
        assert_eq!(normalise_input("  LOOK  "), "look");
    }
}
```

This keeps tests close to the behavior and allows them to exercise private helpers without making internals public just for tests. For test-driven development, start with one narrow failing test, implement the smallest useful behavior, then add the next case.

Good unit test targets include:

- Buffer and frame serialization behavior.
- Z-string decoding and text encoding.
- Dictionary tokenization.
- Arithmetic, branching, and object-table helpers.
- Save/restore state helpers.
- Input parsing rules that do not require a real terminal.

Use table-driven tests when the same behavior has many cases:

```rust
#[test]
fn parses_yes_no_answers() {
    let cases = [
        ("yes", true),
        ("y", true),
        ("no", false),
        ("n", false),
    ];

    for (input, expected) in cases {
        assert_eq!(parse_answer(input), expected, "input: {input}");
    }
}
```

## Interpreter Tests With Fake UI

The interpreter core already depends on the `UI` trait, which should be the main testing seam for `Zmachine`. Tests should provide a small fake UI that feeds scripted input and captures output, instead of driving `TerminalUI` or browser UI code.

Fake-UI tests should cover interpreter behavior that is larger than one helper function but smaller than a whole process run:

- EOF during line input stops the VM without panic.
- Empty line input is still treated as a real empty command, not EOF.
- Read handling updates memory correctly for supported story versions.
- Debug commands and undo/redo behavior preserve expected state.

To make this easy, the Rust crate should expose a normal host-testable library target. `cargo test` should work on the host without compiling wasm-only entry points.

## Stream-Mode Baseline Tests

Add an explicit terminal `--stream` mode for deterministic stdin/stdout tests. This mode is for automation and should not change normal terminal behavior.

In stream mode:

- The CLI flag is explicit: `--stream`.
- EOF from stdin exits cleanly with status code `0`.
- Output is plain text only.
- Crossterm cursor movement, scroll regions, raw mode, status bar updates, and the terminal exit prompt are suppressed.
- Normal terminal behavior remains unchanged unless `--stream` is passed.

Use Rust integration tests for process-level stream checks. These tests should spawn the built binary, pipe scripted input, and compare normalized stdout to committed baselines. Start with one narrow smoke transcript, such as:

```text
encrusted --stream --width 60 tests/minizork.z3
```

with a short command script and a known-good output fixture.

Version 4 and later stories may not produce canonical stream-style baselines because their screen/window behavior does not map cleanly to plain transcript output. Baseline coverage should start with v1-v3 style line-input transcripts and expand cautiously.

## Existing Regression Suite

Keep `tests/regtest.py` and `tests/runtests.sh` as the compatibility regression suite. These tests are valuable because they exercise known Z-machine behavior across existing fixtures.

Use the smallest validation command that matches the change:

- `cargo test` for Rust unit and integration tests.
- `cargo check --bin encrusted` for terminal-only Rust changes.
- `cargo check --lib --target wasm32-unknown-unknown` for wasm-facing Rust changes.
- `npm run test` for the existing full regression flow.

Generated outputs in `build/` and `target/` should not be edited or committed as part of testing work.

## Near-Term Testing Plan

1. Make `cargo test` work on the host by adjusting the Rust library setup.
2. Add function-level unit tests beside the Rust code they protect.
3. Add fake-UI tests for EOF and empty-input behavior.
4. Add explicit `--stream` mode to the terminal binary.
5. Add one Rust integration baseline test that pipes input into `--stream`.
6. Expand baseline fixtures only after the stream format is stable.
