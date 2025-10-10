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
        .parameters({"image_size": NumberField(256), "size": NumberField(10)})
    )


def compute(_, p):
    N = int(p["image_size"])
    size = p["size"]
    print(N)
    image = np.zeros((N, N))

    for x in np.arange(N):
        for y in np.arange(N):
            image[x, y] = float((x / size + y / size) % 2)

    return {"out": image}
