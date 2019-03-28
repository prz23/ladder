#!/usr/bin/env bash

set -e

PROJECT_ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"
EXE_PATH="$PWD/abmatrix"
BASE_PATH="$PWD/"

RUST_LOG=info $EXE_PATH --dev --base-path=$BASE_PATH > abmatrix.log 2>&1 &
