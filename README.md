In order for pyo3 to work correctly (at least on macos), I needed to make sure that a python venv with numpy installed is set for the PYTHONPATH env variable.

in `.cargo/config.toml`
```toml
[env]
PYTHONPATH = "./foray_py/.venv/lib/python3.13/site-packages/"
```
