#!/bin/bash

PID_FILE=$1

echo $$ > $PID_FILE
$SHELL
