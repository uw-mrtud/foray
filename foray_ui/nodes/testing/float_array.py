import numpy as np
from foray import port, UIParameter


def config():
    return {
        "outputs": {"out": (port.Float, [None, None])},
        "parameters": {
            "a": UIParameter.NumberField(0),
        },
    }


def compute(_, p):
    a = float(p["a"])
    out = np.array([[a, a + 1], [a + 1, a]])

    return {"out": out}
