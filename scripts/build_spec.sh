 #!/bin/bash

PROJECT_ROOT=`git rev-parse --show-toplevel`

EXE=$PROJECT_ROOT/target/release/ladder

if [ ! -e $EXE ]; then
    echo "Please build first"
    exit 1
fi

$EXE build-spec --chain=ladder > $PROJECT_ROOT/cli/res/ladder.json