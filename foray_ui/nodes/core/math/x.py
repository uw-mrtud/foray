import numpy as np
from foray import port


def config():
    class out:
        inputs = {"a": port.ArrayComplex, "b": port.ArrayComplex}
        outputs = {"out": port.ArrayComplex}
        parameters = {}

    return out


def compute(input, _):
    a = input["a"]
    b = input["b"]
    out = np.multiply(a, b)
    return {"out": out}
