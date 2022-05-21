#!/bin/bash

set -euo pipefail

if [ -f "Cargo.toml" ]
then
  echo "[VERSION BUMP] Found Cargo.toml and start bumping to $NEXTVERSION"
  sed -i "0,/\(version\s*=\s*\"\)\([0-9]*\.[0-9]*\.[0-9]*\)\(\W*\w*\"\)/s//\1${NEXTVERSION}\3/" Cargo.toml
fi
