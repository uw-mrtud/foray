import numpy as np
from foray import ForayConfig, NumberField, Port, Slider


def config():
    return (
        ForayConfig()
        .outputs(
            {
                "out": Port.array(Port.float, [None]),
            }
        )
        .parameters({"freq": NumberField(10), "length": NumberField(100)})
    )


def compute(_, p):
    n = int(p["length"])
    f = int(p["freq"])

    x = np.linspace(0, 6.28, n)
    y = np.sin(x * f)

    return {"out": y}
