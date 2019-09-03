#!/bin/bash

## TODO build.

# check file exist?

PROJECT_ROOT=`git rev-parse --show-toplevel`


# Save current directory.
pushd . >/dev/null

cd $PROJECT_ROOT/scripts
cp -f $PROJECT_ROOT/target/release/ladder .

# Show version
./ladder --version

# build it docker
docker build . -f ./docker/Dockerfile -t kazee/ladder

# build node docker 
docker build . -f ./docker/node.Dockerfile -t kazee/ladder-node

popd >/dev/null