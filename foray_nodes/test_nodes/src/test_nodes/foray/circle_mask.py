import numpy as np
from foray import ForayConfig, NumberField, Port, Slider


def config():
    return (
        ForayConfig()
        .outputs(
            {
                "out": Port.array(Port.float, [None, None]),
            }
        )
        .parameters({"image_size": NumberField(256), "radius": Slider(0, 256, 1)})
    )


def compute(_, p):
    N = p["image_size"]
    x = np.arange(0, N)
    y = np.arange(0, N)

    half = (N - 1) / 2.0
    radius = p["radius"]
    dist = np.sqrt((x[:, None] - half) ** 2 + (y - half) ** 2)
    out = np.zeros_like(dist)
    # out = out - 1
    out[dist < radius] = 1.0

    return {"out": out}
