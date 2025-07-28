import numpy as np
from foray import port, UIParameter


def config():
    return {
        "inputs": {"in": (port.Float, [None, None])},
        "outputs": {"transposed": (port.Float, [None, None])},
    }


def compute(input, p):
    out = np.transpose(input["in"]) * 2.0
    print(input["in"])
    print(out)
    return {"transposed": out}
