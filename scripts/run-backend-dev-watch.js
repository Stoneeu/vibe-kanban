#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execFileSync, spawn } = require('child_process');

function getCommandExecutable(command) {
  return command?.trim().match(/^[^\s]+/)?.[0];
}

function isCommandAvailable(command) {
  if (!command) {
    return false;
  }

  if (command.includes(path.sep)) {
    return fs.existsSync(command);
  }

  try {
    execFileSync(
      'bash',
      ['-lc', 'command -v "$1" >/dev/null 2>&1', '--', command],
      { stdio: ['ignore', 'ignore', 'ignore'] }
    );
    return true;
  } catch {
    return false;
  }
}

function resolveCompilerPair() {
  const environmentCc = process.env.CC;
  const environmentCxx = process.env.CXX;

  if (environmentCc || environmentCxx) {
    const ccExecutable = getCommandExecutable(environmentCc);
    const cxxExecutable = getCommandExecutable(environmentCxx);

    if (
      ccExecutable &&
      cxxExecutable &&
      isCommandAvailable(ccExecutable) &&
      isCommandAvailable(cxxExecutable)
    ) {
      return {
        cc: environmentCc,
        cxx: environmentCxx,
        source: 'environment',
      };
    }

    console.warn(
      [
        'Ignoring invalid CC/CXX environment overrides.',
        `CC=${environmentCc || '<unset>'}`,
        `CXX=${environmentCxx || '<unset>'}`,
      ].join(' ')
    );
  }

  const fallbackPairs = [
    { cc: 'clang', cxx: 'clang++', source: 'clang' },
    { cc: 'gcc', cxx: 'g++', source: 'gcc' },
    { cc: 'cc', cxx: 'c++', source: 'system-default' },
  ];

  return fallbackPairs.find(
    ({ cc, cxx }) => isCommandAvailable(cc) && isCommandAvailable(cxx)
  );
}

function resolveBindgenExtraClangArgs() {
  if (process.env.BINDGEN_EXTRA_CLANG_ARGS) {
    return process.env.BINDGEN_EXTRA_CLANG_ARGS;
  }

  try {
    const includeDir = execFileSync('gcc', ['-print-file-name=include'], {
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    }).trim();

    if (!includeDir) {
      return undefined;
    }

    if (!fs.existsSync(path.join(includeDir, 'stdarg.h'))) {
      return undefined;
    }

    return `-I${includeDir}`;
  } catch {
    return undefined;
  }
}

const env = {
  ...process.env,
  DISABLE_WORKTREE_CLEANUP: process.env.DISABLE_WORKTREE_CLEANUP || '1',
  RUST_LOG: process.env.RUST_LOG || 'debug',
};

const bindgenExtraClangArgs = resolveBindgenExtraClangArgs();
if (bindgenExtraClangArgs) {
  env.BINDGEN_EXTRA_CLANG_ARGS = bindgenExtraClangArgs;
}

const compilerPair = resolveCompilerPair();
if (!compilerPair) {
  console.error(
    'No supported C/C++ compiler pair found. Install clang or build-essential.'
  );
  process.exit(1);
}

env.CC = compilerPair.cc;
env.CXX = compilerPair.cxx;

if (compilerPair.source !== 'environment') {
  console.log(`Using ${compilerPair.source} toolchain for cargo builds.`);
}

const child = spawn(
  'cargo',
  ['watch', '-w', 'crates', '-x', 'run --bin server'],
  {
    stdio: 'inherit',
    env,
  }
);

child.on('exit', (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }

  process.exit(code ?? 0);
});

child.on('error', (error) => {
  console.error('Failed to start cargo watch:', error);
  process.exit(1);
});
