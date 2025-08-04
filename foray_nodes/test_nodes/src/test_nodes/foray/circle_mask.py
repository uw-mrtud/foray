import numpy as np
from foray import ForayConfig, NumberField, Port, Slider


def config():
    print(Slider(0, 1, 1))
    return (
        ForayConfig()
        .outputs(
            {
                "out": Port.array(Port.float, [None, None]),
            }
        )
        .parameters({"radius": Slider(0, 2, 0.5)})
    )


def compute(_, p):
    N = 256
    x = np.linspace(0, 10, N)
    y = np.linspace(0, 10, N)

    radius = round(float(p["radius"]) * 100) / 10

    dist = (x[:, None] - 5) ** 2 + (y - 5) ** 2
    out = np.zeros_like(dist)
    out = out - 1
    out[dist < radius] = 1.0

    print(out.shape)

    return {"out": out}
