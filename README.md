# wg-web-server

[![Build Status](https://travis-ci.org/gluxon/wg-web-server.svg?branch=develop)](https://travis-ci.org/gluxon/wg-web-server)

Not ready yet. Check back next month.

## macOS Development

These instructions come from Tim Ryan's blog post: https://timryan.org/2018/07/27/cross-compiling-linux-binaries-from-macos.html

```sh
rustup target add x86_64-unknown-linux-gnu
brew tap SergioBenitez/osxct
```

Add the following to `.cargo/config`:

```ini
[build]
target = "x86_64-unknown-linux-gnu"

[target.x86_64-unknown-linux-gnu]
linker = "x86_64-unknown-linux-gnu-gcc"
```

To build:

```sh
export TARGET_CC="x86_64-unknown-linux-gnu-gcc"
cargo build --features "sqlite-bundled"
```
