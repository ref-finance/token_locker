#/bin/bash
VER=1.76.0
rustup toolchain install $VER
rustup default $VER
rustup target add wasm32-unknown-unknown
cargo build -p contract --target wasm32-unknown-unknown --release