# rust-static-fs
Rust static file serve.
It is suitable for serving static built of Single Page Applications (SPA)
# Usage
```
./static-fs-linux-gnu -b :8080
./static-fs-linux-gnu -b 127.0.0.1:8080
./static-fs-linux-gnu -b unix:/tmp/http.sock
./static-fs-linux-gnu -b unix:/tmp/http.sock -f /opt/build/public
```
# Build
```
cargo build --release --target x86_64-unknown-linux-musl
```
