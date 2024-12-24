#!/bin/bash

BASE=$1
NAME=$2

echo $$ > $BASE/$NAME.pid
bash
