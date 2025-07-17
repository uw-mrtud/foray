import numpy as np


def config():
    return {
        "inputs": {"a": "Integer", "b": "Integer"},
        "outputs": {"out": "Integer"},
        "parameters": {},
    }


def compute(input, _):
    a = input["a"]
    b = input["b"]
    out = np.multiply(a, b)
    return {"out": out}
