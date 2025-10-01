import numpy as np
from foray import ForayConfig, NumberField, Port


def config():
    return (
        ForayConfig()
        .outputs(
            {
                "out": Port.array(Port.complex, [None, None]),
            }
        )
        .parameters({"image_size": NumberField(256)})
    )


def compute(_, p):
    N = int(p["image_size"])
    print(N)
    image = np.zeros((N, N), dtype=np.complex128)

    for y in np.arange(N):
        for x in np.arange(N):
            wraps = (x / N) * 2
            scale = np.sin((y / N) * np.pi)
            image[y, x] = (
                np.sin(wraps * 2 * np.pi) + np.cos(wraps * 2 * np.pi) * 1.0j
            ) * scale

    return {"out": image}
