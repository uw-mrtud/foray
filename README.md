
# Building
Instructions for developing foray itself.

Make sure rust is installed, and up to date.

Build with:
```
cargo run
```

## Avoid Unecessary Rebuilds 
To avoid excessive rebuilds caused by different python paths used by cargo directly vs LSP, add an explicit python path in `./.cargo/config.toml`
```toml
[env]
PYO3_PYTHON = "/usr/bin/python"
```
