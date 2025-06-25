#!/usr/bin/env bash
# generate_talea_scaffold.sh
# Creates the talea language project directory tree and placeholder files.

set -euo pipefail

ROOT="."

echo "▶️  Creating talea project scaffold in ./${ROOT}"

# ------------------------------------------------------------------
# 1. Create directories (-p avoids errors if they already exist)
# ------------------------------------------------------------------
mkdir -p "${ROOT}"/{docs,examples,src/{ast,lexer,parser,runtime/ffi},scripts}

# ------------------------------------------------------------------
# 2. Touch files (will overwrite nothing, just ensure they exist)
# ------------------------------------------------------------------
touch \
  "${ROOT}"/.gitignore \
  "${ROOT}"/Cargo.toml \
  "${ROOT}"/README.md \
  "${ROOT}"/docs/language_spec.md \
  "${ROOT}"/docs/user_guide.md \
  "${ROOT}"/examples/hello_world.tea \
  "${ROOT}"/src/{main.rs,lib.rs,cli.rs} \
  "${ROOT}"/src/ast/mod.rs \
  "${ROOT}"/src/lexer/mod.rs \
  "${ROOT}"/src/parser/mod.rs \
  "${ROOT}"/src/runtime/{mod.rs,ffi/{mod.rs,python.rs,r.rs,java.rs}} \
  "${ROOT}"/scripts/{install_dependencies.sh,build.rs}

# ------------------------------------------------------------------
# 3. Make helper script inside the project executable
# ------------------------------------------------------------------
chmod +x "${ROOT}/scripts/install_dependencies.sh"

echo "✅  Scaffold created. Happy hacking!"
