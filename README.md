# Collision Engine
Project to explore common game collision techniques

To run without re-compiling everything:
cargo run --features bevy/dynamic_linking

To run on web:
cargo install wasm-bindgen-cli
cargo install wasm-server-runner
rustup target install wasm32-unknown-unknown

cargo build --release --target wasm32-unknown-unknown

wasm-bindgen --no-typescript --target web     --out-dir ./out/     --out-name "collision_engine"     ./target/wasm32-unknown-unknown/release/collision_engine.wasm

npx serve .