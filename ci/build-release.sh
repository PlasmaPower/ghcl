#!/bin/bash

# See also:
# https://github.com/emk/rust-musl-builder/blob/master/examples/build-release

set -eux

mkdir target/gh-release
if [[ "$(uname -s)" == "Linux" ]]; then
    if [[ "$UID" != "1000" ]]; then
        chmod -R a+w target
    fi
    docker run --rm -it -v "$(pwd)":/home/rust/src ekidd/rust-musl-builder cargo build --release
    zip -j target/gh-release/"$1"-"$2".zip target/x86_64-unknown-linux-musl/release/"$1"
else
    cargo build --release
    zip -j target/gh-release/"$1"-"$2".zip target/release/"$1"
fi
