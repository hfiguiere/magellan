#!/bin/sh

VERSION=$1
DEST=${MESON_BUILD_ROOT}
DIST=$DEST/dist/$VERSION


cd "${MESON_SOURCE_ROOT}"
mkdir -p $DIST

# copying files
cp -rf data $DIST
cp -rf doc $DIST
cp -rf po $DIST
cp -rf src $DIST
cp Cargo.toml $DIST
cp Cargo.lock $DIST
cp meson.build $DIST
cp meson_options.txt $DIST
cp gpsami.doap $DIST
cp AUTHORS $DIST
cp COPYING $DIST
cp NEWS $DIST
cp README $DIST
cp -rf scripts $DIST

#cargo vendor
mkdir $DIST/.cargo
cargo vendor | sed 's/^directory = ".*"/directory = "vendor"/g' > $DIST/.cargo/config
cp -rf vendor $DIST/

# packaging
cd $DEST/dist
tar -cJvf $VERSION.tar.xz $VERSION

