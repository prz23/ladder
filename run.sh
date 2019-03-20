#!/usr/bin/env bash

set -e

PROJECT_ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"
EXE_PATH="$PWD/target/debug/abmatrix"
BASE_PATH="$PWD/target"

RUST_LOG=info $EXE_PATH --chain=local --base-path=$BASE_PATH --key=Alice --validator

#RUST_LOG=info $EXE_PATH --dev --base-path=$BASE_PATH