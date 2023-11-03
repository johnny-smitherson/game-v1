rustup target install wasm32-unknown-unknown
cargo install wasm-bindgen-cli
cargo install wasm-server-runner
cargo install bacon

cargo build -Z build-std=panic_abort,std --target wasm32-unknown-unknown
