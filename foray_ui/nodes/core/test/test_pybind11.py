import numpy as np
from foray import port
import python_example

python_example.add(1, 2)


def config():
    class out:
        inputs = {"a": port.Real, "b": port.Real}
        outputs = {"out": port.Real}
        parameters = {}

    return out


def compute(input, _):
    a = input["a"]
    b = input["b"]
    c = python_example.add(a, b)
    out = np.array([c, c], dtype=np.float64)

    return {"out": out}
