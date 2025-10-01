import numpy as np
from foray import ForayConfig, Port, Slider


def config():
    return (
        ForayConfig()
        .inputs(
            {
                "a": Port.array(Port.float, [None, None]),
            }
        )
        .outputs(
            {
                "out": Port.array(Port.float, [None, None]),
            }
        )
        .parameters({"x-shift": Slider(0, 1, 100), "y-shift": Slider(0, 1, 100)})
    )


def compute(input, p):
    a = input["a"]
    out = np.roll(
        a, (p["x-shift"] * a.shape[0], p["y-shift"] * a.shape[1]), axis=(0, 1)
    )
    return {"out": out}
