# Installation

To get started you will need two things: The foray application, and a python environment that will provide nodes.
We'll cover each of these below.

## Foray
Currently, Foray has been primarily tested on Linux and MacOs.

### MacOs
Foray can be installed via brew. If you don't have brew installed, you can get it [here](https://brew.sh/).

Installing with `brew`:
```bash
brew install uw-mrtud/mrtud/foray
```

### Installing from source
For now, Foray must be manually built for Linux and Windows.

TODO: Manual build instructions

## Python Environment
Foray uses python environemnts to determine what nodes are available.

We'll setup a simple python environment to get started.

Clone the example repository:
```bash
git clone https://github.com/uw-mrtud/foray_example.git
```

The example repository contains a `pyproject.toml` file, which configures the python environment we will build.

## Quick Start
The following steps will get you up and running Foray quickly. Continue below for an explanation of each steps.

We will use `uv` to simplify creating the python virtual environement. It can be installed with brew: `brew install uv`.

```bash
cd foray_example
uv venv # Creates a virtual environment, `./venv`, and installs python packages
foray networks/hello_foray.network
```

Foray should open an example network.

## Creating and updating the python virtual environment

Tools like `uv` exist to simplify creating and modifying python environments.

```bash
uv sync
```
The `uv sync` command  create/updates a virtual environment (located at `./.venv` by default), to match what is specied in the `pyproject.toml` file.

`uv` is nice to use because it avoids many of the manual steps required when modifying python virtual environments.

#### Adding dependencies with UV

For example, if we wanted to add `numpy` as a dependency to our environment:
``` bash
uv add numpy
```

`uv` will automatically update the `pyproject.toml` file to include numpy in its list of dependencies, and will automatically update the virtual environment.

## Manually creating a virtual environment

`uv` isn't required to use Foray.

To manually creat a blank python virtual environment located at `.../foray_example/.venv`:
```bash
cd foray_example
python3 -m venv .venv
```

To manually install the foray_example packge defined in `pyproject.toml`, we need to:

1. activate the virtual environement we just created
2. install the python foray_example python package

```bash
source .venv/bin/activate
pip install -e .
```

The `-e` flag makes the installation "editable" meaning as you update the source code of this package, updates will automatically take effect, and you won't have to re-install the package each time you make edits.
The `.` in the `pip install` command species that the python package is defined in the current directory.

## Launching Foray

To open foray with a specific network:
```bash
foray networks/hello_foray.network
```

If a network file isn't specified, foray will open the network opened most recently

```bash
foray
```

