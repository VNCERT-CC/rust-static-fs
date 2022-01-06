# rust-static-fs
Rust static file serve
# Usage
```
./static-fs-linux-gnu :8080
./static-fs-linux-gnu 127.0.0.1:8080
./static-fs-linux-gnu unix:/tmp/http.sock
./static-fs-linux-gnu unix:/tmp/http.sock /opt/build/public
```
# Build
```
cargo build --release --target x86_64-unknown-linux-musl
```
