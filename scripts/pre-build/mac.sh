#!/bin/sh

curl -o gstreamer-1.0-universal.pkg https://gstreamer.freedesktop.org/data/pkg/osx/$GST_VERSION/gstreamer-1.0-$GST_VERSION-universal.pkg
curl -o gstreamer-1.0-devel-universal.pkg https://gstreamer.freedesktop.org/data/pkg/osx/$GST_VERSION/gstreamer-1.0-devel-$GST_VERSION-universal.pkg

sudo installer -pkg gstreamer-1.0-universal.pkg -target /
sudo installer -pkg gstreamer-1.0-devel-universal.pkg -target /

export PATH="/Library/Frameworks/GStreamer.framework/Versions/1.0/bin:$PATH"
export PKG_CONFIG_PATH="/Library/Frameworks/GStreamer.framework/Versions/1.0/lib/pkgconfig"

echo "PATH=$PATH" >> $GITHUB_ENV
echo "PKG_CONFIG_PATH=$PKG_CONFIG_PATH" >> $GITHUB_ENV