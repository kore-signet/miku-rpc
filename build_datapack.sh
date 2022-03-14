#!/bin/sh
BASE=$PWD
cargo build --release
mkdir -p /tmp/miku-build/lib/lua
riscv64-unknown-linux-musl-objcopy --strip-unneeded target/riscv64gc-unknown-linux-musl/release/libmiku.so /tmp/miku-build/lib/lua/libmiku.so
cd /tmp/miku-build
zip -r miku.zip lib
mv miku.zip $BASE/miku-lua/miku-datapack/data/miku
cd $BASE/miku-lua
rm miku-datapack.zip
cd miku-datapack
zip -r miku-datapack.zip pack.mcmeta data
mv miku-datapack.zip ..
