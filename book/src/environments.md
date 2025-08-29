## Environments

## Creating and updating the python virtual environment

Tools like `uv` exist to simplify creating and modifying python environments.

```bash
uv sync
```
The `uv sync` command  create/updates a virtual environment (located at `./.venv` by default), to match what is specified in the `pyproject.toml` file.

`uv` is nice to use because it avoids many of the manual steps required when modifying python virtual environments.

#### Adding dependencies with UV

For example, if we wanted to add `numpy` as a dependency to our environment:
``` bash
uv add numpy
```

`uv` will automatically update the `pyproject.toml` file to include numpy in its list of dependencies, and will automatically update the virtual environment.

## Manually creating a virtual environment

`uv` isn't required to use Foray.

To manually create a blank python virtual environment located at `.../foray_example/.venv`:
```bash
cd foray_example
python3 -m venv .venv
```

To manually install the foray_example package defined in `pyproject.toml`, we need to:

1. activate the virtual environment we just created
2. install the python foray_example python package

```bash
source .venv/bin/activate
pip install -e .
```

The `-e` flag makes the installation "editable" meaning as you update the source code of this package, updates will automatically take effect, and you won't have to re-install the package each time you make edits.
The `.` in the `pip install` command species that the python package is defined in the current directory.

