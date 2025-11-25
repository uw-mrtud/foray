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
        .parameters(
            {
                "freq": NumberField(2),
                "phase": Slider(0.0, 6.28, 18),
                "length": NumberField(100),
            }
        )
    )


def compute(_, p):
    n = int(p["length"])
    f = int(p["freq"])
    ph = int(p["phase"])

    x = np.linspace(0, 6.28, n)
    y = np.sin(x * f - ph)

    return {"out": y}
