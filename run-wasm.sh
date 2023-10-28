cargo build  --profile release-wasm  --no-default-features --features wasm  --target wasm32-unknown-unknown && wasm-server-runner target/wasm32-unknown-unknown/release-wasm/game-v3.wasm

