import numpy as np
from foray import port, ui


def config():
    class out:
        inputs = {}
        outputs = {"out": port.ArrayReal}
        parameters = {"radius": ui.Slider}

    return out


def compute(_, p):
    N = 256
    x = np.linspace(0, 10, N)
    y = np.linspace(0, 10, N)

    radius = round(float(p["radius"]) * 10)

    dist = (x[:, None] - 5) ** 2 + (y - 5) ** 2
    out = np.zeros_like(dist)
    out[dist < radius] = 1.0

    return {"out": out}
