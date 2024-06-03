#!/usr/bin/env bash

root=$(git rev-parse --show-toplevel)
pushd "$root"
trap "popd" EXIT

cargo build --release --target wasm32-unknown-unknown

cd ./happs
for happ in $(ls); do
    exit 1
    cd $happ
    hc dna pack .
    hc app pack .
    cd ..
done
