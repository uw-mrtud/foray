import numpy as np
from foray import port


def config():
    class out:
        inputs = {
            "a": port.ArrayReal,
            "b": port.ArrayReal,
        }
        outputs = {"out": port.ArrayReal}
        parameters = {}

    return out


def compute(input, _):
    a = input["a"]
    b = input["b"]
    out = np.cross(a, b)
    return {"out": out}
