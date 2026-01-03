#!/bin/bash
TARGET=arm-unknown-linux-musleabihf
WORKING_DIR=geo-rs
BINARY=target/arm-unknown-linux-musleabihf/debug/geo-rs
CALIBRATOR=target/arm-unknown-linux-musleabihf/debug/calibrate
DESTINATION_BIN="pi@192.168.2.2:/home/pi/geo-rs"
DESTINATION_CALIBRATE="pi@192.168.2.2:/home/pi/geo-calibrate"

echo "moving to working directory..."
cd $WORKING_DIR
ls -lah

echo "running cargo clean"
cargo clean 

echo "compiling binary for target $TARGET"
cargo build --target $TARGET --verbose

echo "deploying..."
scp $BINARY $DESTINATION_BIN
scp $CALIBRATOR $DESTINATION_CALIBRATE

echo "done."
