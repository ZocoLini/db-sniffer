#!/bin/bash

INSTALLATION_DIR="$HOME/.local/bin"
BUILD_BIN="target/release/cli"

cargo build --release -p cli --bin cli

mkdir $INSTALLATION_DIR
mv $BUILD_BIN "$INSTALLATION_DIR/sniffer"