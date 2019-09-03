#!/bin/bash

set -e

PROJECT_ROOT=`git rev-parse --show-toplevel`
PUBLISH=publish

# Build wasm
$PROJECT_ROOT/scripts/build.sh

# Save current directory.
pushd . >/dev/null

# Build target
cd $PROJECT_ROOT
echo "Build ladder"
cargo build --release

# Copy files 
FILES=(
    "target/release/ladder"
    "scripts/ladder.sh"
    "cli/res/ladder.json"
)

mkdir -p ./$PUBLISH
for FILE in "${FILES[@]}"
do
    cp -f $PROJECT_ROOT/$FILE ./$PUBLISH
done

# Make package
cd $PROJECT_ROOT/$PUBLISH
version=$(./ladder --version | sed  's/^ladder //')
tar -cf $version.tar ./ladder ./ladder.sh ./ladder.json

echo "Package '$version.tar' is ready."
# Restore initial directory.
popd >/dev/null