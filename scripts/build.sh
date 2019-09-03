#!/usr/bin/env bash

set -e

PROJECT_ROOT=`git rev-parse --show-toplevel`
ROOT=$PROJECT_ROOT

SRCS=(
	"runtime/wasm"
)

export CARGO_INCREMENTAL=0

bold=$(tput bold)
normal=$(tput sgr0)

# Save current directory.
pushd . >/dev/null

cd $ROOT

for SRC in "${SRCS[@]}"
do
  echo "${bold}Building wasm binaries in $SRC...${normal}"
  cd "$PROJECT_ROOT/$SRC"

  ./build.sh

  cd - >> /dev/null
done

# Restore initial directory.
popd >/dev/null
