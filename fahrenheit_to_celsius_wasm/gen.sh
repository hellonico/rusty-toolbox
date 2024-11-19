https://rustwasm.github.io/wasm-pack/installer/

# don't forget
# https://stackoverflow.com/questions/74005040/why-i-cannot-find-the-compiled-wasm-file
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --out-dir ../fahrenheit_to_celsius/static --target web ../target/wasm32-unknown-unknown/release/frontend.wasm


```
brew install cargo-binstall
cargo binstall wasm-bindgen-cli
```