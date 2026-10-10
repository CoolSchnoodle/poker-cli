#!/bin/sh
rm -r target
cargo build --release
mkdir tmp
find target/release -maxdepth 1 -type f | grep -v '\.' | xargs -I % mv % tmp/engine
rm -r target
chmod +x engine
