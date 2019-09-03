#!/bin/bash

# directory structure
# |--ladder.sh
# |--ladder  # for native start if exist
# |--nodes   # for storage (native and docker)
#     |--test
#         |--chains
#     |--test2
#         |--chains
#     |--logs
#         |--test.log
#         |--test2.log

#set -e

DOCKER_IMAGE=kazee/ladder-node
EXE_NAME=ladder
PROJECT_ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"
BASE_PATH="$PROJECT_ROOT/nodes"
EXE_PATH="$PROJECT_ROOT/$EXE_NAME"
LOGS_PATH="$BASE_PATH/logs"
DOCKER_IMAGE="ladder-node"
CONTAINER_NAME="ladder"

usage() {
    cat <<EOF
Usage: $SCRIPT <command> <node> [options]
where <command> is one of the following:
    { help | start | stop | clean | logs | update | top }
Run \`$SCRIPT help\` for more detailed information.
EOF
}

# INFORMATIONAL COMMANDS
help() {
    cat <<EOF
Usage: $SCRIPT <command> <node> [options]
This is the primary script for controlling the $SCRIPT node.
 INFORMATIONAL COMMANDS
    help
        Print help.
 NODE CONTROL COMMANDS
    start <node>
        Starts the $SCRIPT node in the background. If the node is already
        started, you will get the message "Node is already running!" If the
        node is not already running, no output will be given.
    stop <node>
        Stops the running $SCRIPT node. Prints "ok" when successful.  When
        the node is already stopped or not responding, prints:
        "Node 'NODE_NAME' not responding to pings."
    restart <node>
        Stops and then starts the running $SCRIPT node. Prints "ok"
        when successful.  When the node is already stopped or not
        responding, prints: "Node 'NODE_NAME' not responding to
        pings."
 DIAGNOSTIC COMMANDS
    top
        Prints all node information similar
        to the information provided by the \`top\` command.
    logs <node>
        Fetch the log of the node.
 SCRIPTING COMMANDS
    clean <node>
        Clean the node's data and logs, which actually move that data and logs
        into backup directory. Prints the specified backup commands. When the
        node is running, prints: "Node is already running!"
    update
        Update docker images. This need sudo.
EOF

}

INIT_CMD='sleep infinity'

env() {
    # Check if need to run in docker?
    isDockerEnv=false

    if [ ! -e $EXE_PATH ]; then
        isDockerEnv=true
    else
        # Get the version 
        canRunNative=$($EXE_PATH --version | grep "^ladder.*")
        if test -z "$canRunNative" ; then
            isDockerEnv=true
        fi
    fi

    if [ $isDockerEnv == true ]; then
        #echo "Use docker only"
        # Test docker
        if ! test -x docker ; then
            echo "Please install docker firstly, use 'apt install docker.io' command"
            exit 1
        fi

        # run instance in one docker or multi docker? impl single
        # nohup docker run -p 30335:30333 -p 9966:9933 -p 9977:9944 -v $PWD/data:/data --name kusama parity/polkadot:v0.5.0 --validator --name "laddernetwork" --base-path="/data" --ws-external --rpc-cors=all > $PWD/kusama.log 2>&1 &

        if ! docker ps | grep "${CONTAINER_NAME}" > '/dev/null' 2>&1; then
            # remove dead container
            docker rm "${CONTAINER_NAME}" > '/dev/null' 2>&1

            # run new container
            docker run -d \
                --net=host \
                --volume `pwd`/nodes:/chain/nodes  \
                --name ${CONTAINER_NAME} \
                ${DOCKER_IMAGE} \
                /bin/bash -c "${INIT_CMD}"
            sleep 3
        fi

        local background=false
        if [[ $1 == "start" ]] || [[ $1 == "restart" ]]; then
            background=true
        fi

        if [ $background = true ]; then
            docker exec -id ${CONTAINER_NAME} ./ladder.sh "$@"
        else
            docker exec -it ${CONTAINER_NAME} ./ladder.sh "$@"
        fi
        exit 0
    fi
}


start() {
    if [ $NODE_NAME = "dev" ]; then
        RUST_LOG='info' $EXE_PATH --dev --base-path=$NODE_PATH --name=$NOED_NAME >> $LOG_FILE 2>&1 & 
    else
        RUST_LOG='info' $EXE_PATH --chain=ladder --base-path=$NODE_PATH --name=$NODE_NAME --bootnodes /ip4/47.56.107.144/tcp/30333/p2p/QmXS53cQyDRT7RaXiKYLjfkX8xSc9pBDPohDh1F3HxzjAz --validator --telemetry-url ws://telemetry.polkadot.io:1024 >> $LOG_FILE 2>&1 & 
    fi

    # TODO check pid.
    echo "Start node: $NODE_NAME"
    exit 0
}


restart() {
    stop
    start
}

# stop node
stop() {
    local pid=$(ps -ef | grep $EXE_NAME | grep $NODE_PATH | grep -v grep | awk '{print $2}')
    if [ $pid ]; then
        kill -9 $pid
        echo "Killed $pid, name: $NODE_NAME"
    else
        echo "No such process"
    fi 
}

# clean database
clean() {
    if [ ! -d $NODE_PATH ]; then
        echo "No such node directory: ${NODE_NAME}"
        exit 1
    fi

    rm -r $NODE_PATH
    rm $LOG_FILE
    echo "clean $NODE_PATH"
}

logs() {
    tail -f $LOG_FILE
}

update() {
    # update docker
    sudo docker pull $DOCKER_IMAGE
}

top() {
    ps -ef | \
    grep $EXE_NAME | \
    awk 'BEGIN {print "user \tpid \ttime \tname"} /--base-path/{print $1 "\t" $2 "\t" $5 "\t" $10}' | \
    sed "s/--base.*nodes\///g"
}

sudo() {
    set -o noglob

    if [ "$(whoami)" == "root" ] ; then
        "$@"
    else
        /usr/bin/sudo "$@"
    fi
    set +o noglob
}

ensureDir() {
    mkdir -p nodes
    mkdir -p nodes/logs
}

dealwith() {
    local command="$1"
    case "${command}" in
        help)
            help
            ;;
        start)
            start
            ;;
        stop)
            stop
            ;;
        restart)
            restart
            ;;
        clean)
            clean
            ;;
        logs)
            logs
            ;;
        update)
            update
            ;;
        top)
            top
            ;;
        *)
            usage
            ;;
    esac
}

main() {
    # Mock directory
    ensureDir

    # Check runing environment
    env "$@"

    # Commands not depend on node name
    local indie=( help usage update top )
    local command=$1
    if [[ "${indie[*]}" =~ $command ]]; then
        dealwith "$@"
        exit 0
    fi 

    # Commands depend on node name
    if [ $# -lt 2 ]; then
        usage
        exit 1
    fi

    NODE_NAME=$2
    NODE_PATH="$BASE_PATH/$NODE_NAME"
    LOG_FILE="$LOGS_PATH/$NODE_NAME.log"

    dealwith "$@"

    exit 0
}

main "$@"