#!/bin/sh

cargo build --release --target wasm32-unknown-unknown --manifest-path zomes/remote_ping_zome/coordinator/Cargo.toml
hc app pack zomes/remote_ping_zome/ --recursive
