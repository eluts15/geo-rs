#!/bin/bash
TARGET=arm-unknown-linux-musleabihf
WORKING_DIR=geo-rs
FILE_PATH=target/arm-unknown-linux-musleabihf/debug/geo-rs
DESTINATION="pi@192.168.2.2:/home/pi/geo-rs"

echo "moving to working directory..."
cd $WORKING_DIR
ls -lah

echo "running cargo clean"
cargo clean 

echo "compiling binary for target $TARGET"
cargo build --target $TARGET --verbose

echo "deploying..."
scp $FILE_PATH $DESTINATION

echo "done."
