#!/bin/bash

while read p; do
  bash -c "$p"
done < "$1"

bash
