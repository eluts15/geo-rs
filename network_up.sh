#!/bin/bash
ADDRESS=192.168.2.1/24
INTERFACE=enp0s31f6
# Assign yourself a static IP on the ethernet interface
sudo ip addr add $ADDRESS dev $INTERFACE
sudo ip link set $INTERFACE up
