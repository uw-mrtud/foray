
# Building
Instructions for developing foray itself.

Make sure rust is installed, and up to date.

Build with:
```
cargo run
```

## Development Python Environment
In order for pyo3 to work correctly (at least on macos), I needed to make sure that a python venv with numpy installed is set for the PYTHONPATH env variable.

in `.cargo/config.toml`
```toml
[env]
PYTHONPATH = "./foray_py/.venv/lib/python3.13/site-packages/"
```

To avoid excessive rebuilds caused by different python paths used by cargo directly vs LSP, add an explicit python path
```toml
PYO3_PYTHON = "/home/john/projects/foray_data_model/foray_py/.venv/bin/python"
```
