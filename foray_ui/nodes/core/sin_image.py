import numpy as np
from foray import port, UIParameter


def config():
    return {
        "outputs": {"out": (port.Float, [None, None])},
        "parameters": {
            "frequency": UIParameter.Slider(0.01, 10, 1),
            "vertical": UIParameter.CheckBox(False),
        },
    }


def compute(_, p):
    N = 256
    y = np.linspace(0, 10, N)

    frequency = float(p["frequency"])

    out = np.tile(np.sin(y[:, None] * frequency), N)

    print(out.shape)

    return {"out": out}
