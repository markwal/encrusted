const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const mode = process.argv[2] === 'release' ? 'release' : 'debug';
const root = path.resolve(__dirname, '..');
const pkgDir = path.join(root, 'pkg');
const rawWasm = path.join(
  root,
  'target',
  'wasm32-unknown-unknown',
  mode,
  'web.wasm'
);

function run(command, args) {
  const result = spawnSync(command, args, {
    cwd: root,
    stdio: 'inherit',
  });

  if (result.error) {
    if (result.error.code === 'ENOENT') {
      const printable = [command].concat(args).join(' ');
      throw new Error(`Missing required command: ${printable}`);
    }

    throw result.error;
  }

  if (result.status !== 0) {
    process.exit(result.status);
  }
}

fs.mkdirSync(pkgDir, { recursive: true });

run('cargo', [
  'build',
  '--lib',
  '--target',
  'wasm32-unknown-unknown',
  ...(mode === 'release' ? ['--release'] : []),
]);

const wasmBindgenArgs = [
  rawWasm,
  '--out-dir',
  pkgDir,
  '--target',
  'bundler',
];

if (mode === 'debug') {
  wasmBindgenArgs.push('--debug', '--keep-debug');
}

run('wasm-bindgen', wasmBindgenArgs);
