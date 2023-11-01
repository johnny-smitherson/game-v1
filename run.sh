set -ex

# cargo fmt
# cargo clippy #  -- -D warnings
cargo build
cargo run