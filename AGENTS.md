# AGENTS.md

This file gives coding agents a fast orientation for working in this repository.

## Project Summary

`encrusted` is a Z-machine interpreter for Infocom-style text adventures. This fork adds an Electron wrapper around the original Rust/WebAssembly project so it can run as a desktop app as well as in the browser and terminal.

The repo is a mixed Rust + JavaScript codebase:

- Rust implements the interpreter and both UI entry points.
- WebAssembly is built from the Rust library target.
- React/webpack powers the browser UI.
- Electron packages the web app for desktop use.

## Top-Level Layout

- `src/rust/`: core interpreter and Rust UI entry points.
- `src/js/`: React app, Redux logic, worker bridge, Electron renderer entry.
- `src/electron/`: Electron main-process code.
- `src/assets/` and `assets/`: static assets used by the app/package.
- `tests/`: Z-machine regression fixtures and the `regtest.py` runner.
- `build/`: generated web bundle output.
- `target/`: Cargo build output.
- `webpack.*.js`: webpack configs for dev, production, and Electron packaging.
- `Cargo.toml`: Rust targets and dependencies.
- `package.json`: JS/Electron scripts and packaging config.

## Important Entry Points

- Terminal binary: `src/rust/main.terminal.rs`
- WebAssembly library: `src/rust/main.web.rs`
- Core interpreter: `src/rust/zmachine.rs`
- Web UI bridge: `src/rust/ui_web.rs`
- Browser renderer entry: `src/js/index.js`
- Electron renderer entry: `src/js/electron-index.js`

## Build And Check Commands

Use the smallest command that validates your change.

- `cargo check --bin encrusted`
  Checks the terminal build.
- `cargo check --lib --target wasm32-unknown-unknown`
  Checks the wasm/web build.
- `npm run build:debug`
  Builds the wasm artifact into `build/web.wasm`.
- `npm run release`
  Creates the production web bundle and release wasm build.
- `npm run bundle:electron`
  Bundles the Electron renderer/main packaging assets.
- `npm run electron`
  Launches the Electron app.
- `npm run dev`
  Starts the webpack dev server on port `8000`.

## Tests

- `npm run test`
  Builds `target/debug/encrusted` and runs the regression suite in `tests/runtests.sh`.

Notes:

- The test runner shell script uses `bash` and `python`, so it assumes those are available.
- The project README still notes that the web build expects Rust nightly plus the `wasm32-unknown-unknown` target.
- There is no obvious repo-wide formatter script in `package.json`; keep edits stylistically consistent with surrounding code.

## Working Conventions

- Prefer narrow, local changes. Rust and UI code are coupled through the wasm bridge, so cross-layer changes should be deliberate.
- Do not edit generated output in `build/` or `target/` unless the task is explicitly about generated artifacts.
- Keep JS changes compatible with the existing stack in `package.json`:
  React 17, React Router 5, Redux, webpack 5, Electron 12.
- Preserve the existing plain-JS style in `src/js/`; this repo does not appear to use TypeScript.
- Preserve the existing Rust style and crate structure; the interpreter logic is centralized in `src/rust/zmachine.rs`.

## Practical Tips For Future Agents

- When creating a shell to run commands, always set the current working directory to the repo root
- When running PowerShell commands, use `login: true` so the user's PowerShell profile loads and the default `fnm` Node environment is available. After the shell starts, explicitly change the current working directory back to the `encrusted` working tree before running repo commands.
- If you touch wasm-facing Rust code, also sanity-check the related JS bridge code in `src/js/worker.js`, `src/js/WorkerController.js`, and reducer/middleware files.
- If you touch packaging or desktop behavior, inspect both `package.json` and `src/electron/`.
- If you change interpreter behavior, prefer running the regression suite before finishing.
