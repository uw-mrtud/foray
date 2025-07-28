import numpy as np


def config():
    return {
        "inputs": {"a": ("Float", [None, None]), "b": ("Float", [None, None])},
        "outputs": {"out": ("Float", [None, None])},
        "parameters": {},
    }


def compute(input, _):
    a = input["a"]
    b = input["b"]
    out = np.multiply(a, b)
    return {"out": out}
