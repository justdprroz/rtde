#!/bin/bash

if ! test -f src/config.rs; then
  cp src/config-example.rs src/config.rs
fi

cargo install --path .
