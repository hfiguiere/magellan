#!/bin/sh

export CARGO_HOME=$1/target/cargo-home
export LOCALEDIR="$3"
export APP_ID="$4"
export VERSION="$5"
export PROFILE="$6"

if [[ "$PROFILE" == "Devel" ]]
then
    echo "DEBUG MODE"
    cargo build -p gpsami && cp $1/target/debug/gpsami $2
else
    echo "RELEASE MODE"
    cargo build --release -p gpsami && cp $1/target/release/gpsami $2
fi

