#!/bin/bash
set -e

VERSION="0.1.0"
ARCH="amd64"
PKG_NAME="snoopy"

cargo +nightly build --release

mkdir -p "build/DEBIAN"
mkdir -p "build/usr/local/bin"
mkdir -p "build/lib/systemd/system"

cp target/release/snoopy build/usr/local/bin/
cp debian/snoopy@.service build/lib/systemd/system/
cp debian/control build/DEBIAN/

if [ -f debian/postinst ]; then
    cp debian/postinst build/DEBIAN/
    chmod 755 build/DEBIAN/postinst
fi

if [ -f debian/prerm ]; then
    cp debian/prerm build/DEBIAN/
    chmod 755 build/DEBIAN/prerm
fi

dpkg-deb --build build "${PKG_NAME}_${VERSION}_${ARCH}.deb"

echo "Package built: ${PKG_NAME}_${VERSION}_${ARCH}.deb"
