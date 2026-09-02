#!/bin/sh
cargo bundle --release
cp -r target/release/bundle/osx/*.app ~/Applications/