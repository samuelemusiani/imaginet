#!/bin/bash

while read p; do
  bash -c "$p"
done < "/tmp/sconf_$1"

bash
