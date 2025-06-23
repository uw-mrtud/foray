import numpy as np
from foray import port, ui


def config():
    class out:
        inputs = {}
        outputs = {"out": port.Dynamic}
        parameters = {
            "X": ui.Slider,
        }

    return out


def compute(_, parameters):
    x = max(0, min(20, round(float(parameters["X"]) * 100.0)))

    return {"out": np.zeros(tuple(3 for _ in range(0, x)))}
