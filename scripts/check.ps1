$ErrorActionPreference = "Stop"

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test
npm.cmd run build --prefix apps/mineia-launcher
