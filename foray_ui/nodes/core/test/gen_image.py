import numpy as np


def config():
    class out:
        inputs = {"freq": "Real"}
        outputs = {"out": "Real2d"}

    return out


def compute(input):
    x = np.linspace(0, 10, 64)
    y = np.linspace(0, 10, 64)

    freq = input["freq"]

    out = np.sin(2.0 * freq * x[:, None]) * np.cos(freq * y)

    return out
