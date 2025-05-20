#!/bin/bash

PID_FILE=$1
CONF_FILE=$2

echo $$ > "$PID_FILE"

if [[ "x$CONF_FILE" != "x" ]]; then
  bash "$CONF_FILE"
fi

$SHELL
