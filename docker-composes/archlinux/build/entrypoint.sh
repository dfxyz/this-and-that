#!/usr/bin/bash

if [[ ${#@} == 0 ]]; then
    exec "/usr/bin/zsh"
fi

if [[ $1 == "-" ]]; then
    echo "keeping alive..."
    trap "done=1" SIGINT SIGTERM SIGQUIT
    done=0
    while [[ $done == 0 ]]; do
        sleep 1
    done
    exit 0
fi

exec "$@"
