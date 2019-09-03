#!/bin/bash

## TODO build.

PROJECT_ROOT=`git rev-parse --show-toplevel`

# Save current directory.
pushd . >/dev/null

cd $PROJECT_ROOT/scripts
cp -f $PROJECT_ROOT/target/release/ladder .
cp -f $PROJECT_ROOT/cli/res/ladder.json .

# Show version
./ladder --version

# Build it docker
docker build . -f ./docker/Dockerfile -t kazee/ladder

# Build node docker 
docker build . -f ./docker/node.Dockerfile -t kazee/ladder-node

# Remove file
rm -f ./ladder
rm -f ./ladder.json

popd >/dev/null