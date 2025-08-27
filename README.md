# chlaty-hianime
- This is a source code to build shared library for `chlaty-core` dynamic linking.
- You can get a precompiled shared library from [Build Workflows]((./actions)) or [Releases](./releases).
- Write test logic inside `src/test.rs`. Then run the test using:
```bash
cargo test -- --nocapture
```
- To build a release use:
```bash
cargo build --release
```
- This will compile a shared library for your platform.
```
target/release/
- *.dll -> Windows
- *.so -> Linux (This also support android)
- *.dylib -> MacOS
```