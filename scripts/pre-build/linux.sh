#!/bin/sh

sudo apt-get update
sudo apt-get remove -y libunwind-14-dev
sudo apt-get install -y libsystemd-dev libwebp-dev libxcb-shape0-dev libxcb-xfixes0-dev libgtk-3-dev libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly gstreamer1.0-libav libgstrtspserver-1.0-dev libges-1.0-dev