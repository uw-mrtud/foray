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
Foray uses python environments for performing computation, and to determine what nodes are available.
We'll setup a simple python environment to get started.

Clone the example repository:
```bash
git clone https://github.com/uw-mrtud/foray_example.git
```

The example repository contains a `pyproject.toml` file, which configures the python environment we will build. It also includes a few example nodes.

## Quick Start

For this example we will use `uv` to simplify creating the python virtual environment. 
`uv` is not required to use `foray`, any method of creating python virtual environments.
To instal with brew: `brew install uv`.

```bash
cd foray_example
uv sync
foray ./networks/fft_lowpass.network
```

`uv sync` Creates a virtual environment, `./venv`, and installs python packages into the environment.

The last commands opens Foray with a specific network.

