#!/usr/bin/env bash

root=$(git rev-parse --show-toplevel)
pushd "$root"
trap "popd" EXIT

cargo build --release --target wasm32-unknown-unknown

cd ./happs
for happ in $(ls); do
    pushd $happ
    hc dna pack .
    hc app pack .
    popd
done
