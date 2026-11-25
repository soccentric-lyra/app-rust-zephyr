#!/usr/bin/env bash
#
# test-host.sh - run the HWLINK protocol tests on the host
#
# Author:  Sandesh Ghimire <sandesh@soccentric.com>
# Usage:   scripts/test-host.sh
#
# The test crate includes src/protocol/hwlink/frame.rs and runs its unit tests
# with std in the Rust image, so the pinned toolchain is the same one the
# firmware uses.
#

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
DEV="$ROOT/scripts/docker-dev.sh"
cd "$ROOT"

"$DEV" run env CARGO_HOME=/work/.cargo-home \
    cargo test --target x86_64-unknown-linux-gnu \
    --manifest-path protocol/hwlink-tests/Cargo.toml "$@"
