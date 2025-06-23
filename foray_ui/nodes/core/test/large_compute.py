import numpy as np
from foray import port


def config():
    class out:
        inputs = {"radius": port.Real}
        outputs = {"out": port.ArrayReal}
        parameters = {}

    return out


def compute(input, _):
    x = np.linspace(0, 10, 2048)
    y = np.linspace(0, 10, 2048)

    radius = input["radius"]

    dist = (x[:, None] - 5) ** 2 + (y - 5) ** 2
    out = np.zeros_like(dist)
    out[dist < radius] = 1.0

    return {"out": out}
