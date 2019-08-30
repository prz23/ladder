#!/bin/bash

# directory structure
# |--ladder.sh
# |--nodes   # for storage
#     |--ladder  # for native start if exist
#     |--test
#         |--chains
#     |--test2
#         |--chains
#     |--logs
#         |--test.log
#         |--test2.log

set -e

DOCKER_IMAGE=kazee/ladder-node
EXE_NAME=ladder
PROJECT_ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"
BASE_PATH="$PROJECT_ROOT/nodes"
EXE_PATH="$BASE_PATH/$EXE_NAME"
LOGS_PATH="$BASE_PATH/logs"

usage() {
    cat <<EOF
Usage: $SCRIPT <command> <node> [options]
where <command> is one of the following:
    { help | start | stop | clean | logs | update }
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
        You are here.
 NODE CONTROL COMMANDS
    setup <node>
        Ensuring the required runtime environment for $SCRIPT node, like
        RabbitMQ service. You should run this command at the first time
        of running $SCRIPT node.
    start <node>
        Starts the $SCRIPT node in the background. If the node is already
        started, you will get the message "Node is already running!" If the
        node is not already running, no output will be given.
    stop <node> [debug] [mock]
        Stops the running $SCRIPT node. Prints "ok" when successful.  When
        the node is already stopped or not responding, prints:
        "Node 'NODE_NAME' not responding to pings."
    restart <node>
        Stops and then starts the running $SCRIPT node. Prints "ok"
        when successful.  When the node is already stopped or not
        responding, prints: "Node 'NODE_NAME' not responding to
        pings."
 DIAGNOSTIC COMMANDS
    ping <node>
        Checks that the $SCRIPT node is running. Prints "pong" when
        successful.  When the node is stopped or not responding, prints:
        "Node 'NODE_NAME' not responding to pings."
    top <node>
        Prints services processes information similar
        to the information provided by the \`top\` command.
    stat <node> (deprecated, use 'top' instead)
    logs <node> <service>
        Fetch the logs of the specified service.
 SCRIPTING COMMANDS
    clean <node>
        Clean the node's data and logs, which actually move that data and logs
        into backup directory. Prints the specified backup commands. When the
        node is running, prints: "Node is already running!"
EOF

}

# example: start name 0x0000 
start() {
    # find local?
    if [ -e $EXE_PATH ]; then
        local LOG_FILE="$LOGS_PATH/$NODE_NAME.log"
        local NODE_PATH="$BASE_PATH/$NODE_NAME"

        # Start node
        # TODO to check env, if support to run with native, then run in native, else 
        if [ $NODE_NAME = "dev" ]; then
            RUST_LOG='info' $EXE_PATH --dev --base-path=$NODE_PATH >> $LOG_FILE 2>&1 & 
        else
            RUST_LOG='info' $EXE_PATH --chain=ladder --name=$NODE_NAME --base-path=$NODE_PATH --bootnodes /ip4/47.56.107.144/tcp/30333/p2p/QmXS53cQyDRT7RaXiKYLjfkX8xSc9pBDPohDh1F3HxzjAz --validator --telemetry-url ws://telemetry.polkadot.io:1024 >> $LOG_FILE 2>&1 & 
        fi

        echo "Start node: $NODE_NAME"
        exit 0
    fi

    echo "not exist"
    # use docker only
    # test docker
}

# stop node
stop() {
    # pkill ladder
    local SP="nodes/$NODE_NAME"
    local pid=$(ps -ef | grep $EXE_NAME | grep $SP | grep -v grep | awk '{print $2}')
    echo $pid
    if [ $pid ]; then
        kill -9 $pid
        echo "killed $pid"
    else
        echo "No such process"
    fi 
}

# clean database
clean() {
    local NODE_PATH="$BASE_PATH/$NODE_NAME"
    local LOG_FILE="$LOGS_PATH/$NODE_NAME.log"
    if [ ! -d $NODE_PATH ]; then
        echo "No such node directory: ${NODE_NAME}"
        exit 1
    fi

    rm -r $NODE_PATH
    rm $LOG_FILE
    echo "clean $NODE_PATH"
}

logs() {
    local LOG="$LOGS_PATH/$NODE_NAME.log"
    tail -f $LOG
}

update() {
    # update docker
    sudo docker pull $DOCKER_IMAGE
}

top() {
    ps -e | grep -w $EXE_NAME
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
    if [ ! -d $BASE_PATH ]; then
        mkdir nodes
    fi

    if [ ! -d $LOGS_PATH ]; then 
        mkdir nodes/logs
    fi
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
    # echo $PROJECT_ROOT
    # echo $EXE_PATH
    # echo $BASE_PATH

    ensureDir

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

    dealwith "$@"

    exit 0
}

main "$@"