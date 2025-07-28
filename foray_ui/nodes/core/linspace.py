import numpy as np
from foray import port, UIParameter


def config():
    return {
        "outputs": {"out": (port.Float, [None])},
        "parameters": {
            "start": UIParameter.NumberField(0),
            "stop": UIParameter.NumberField(100),
            "num_steps": UIParameter.NumberField(50),
        },
    }


def compute(_, p):
    start = float(p["start"])
    stop = float(p["stop"])
    num_steps = int(p["num_steps"])
    x = np.linspace(start, stop, num_steps)

    return {"out": x}
