#!/usr/bin/env bash
# generate_talea_scaffold.sh
# Creates the talea language project directory tree and placeholder files.

set -euo pipefail

ROOT="."

echo "▶️  Creating talea project scaffold in ./${ROOT}"

# ------------------------------------------------------------------
# 1. Create directories (-p avoids errors if they already exist)
# ------------------------------------------------------------------
mkdir -p "${ROOT}"/{docs,examples,scripts}

# ------------------------------------------------------------------
# 2. Touch files (will overwrite nothing, just ensure they exist)
# ------------------------------------------------------------------
touch \
  "${ROOT}"/.gitignore \
  "${ROOT}"/README.md \
  "${ROOT}"/docs/language_spec.md \
  "${ROOT}"/docs/user_guide.md \
  "${ROOT}"/examples/hello_world.tea \
  "${ROOT}"/scripts/{install_dependencies.sh,build.rs}

# ------------------------------------------------------------------
# 3. Make helper script inside the project executable
# ------------------------------------------------------------------
chmod +x "${ROOT}/scripts/install_dependencies.sh"

echo "✅  Scaffold created. Happy hacking!"
