import numpy as np
from foray import ForayConfig, NumberField, Port, Slider


def config():
    return ForayConfig().outputs(
        {
            "out": Port.array(Port.float, [None, None]),
        }
    )


def compute(_, p):
    data = np.zeros((4, 5, 2))

    data[0, 0, 0] = 1
    data[1, 1, 0] = 1
    data[2, 2, 0] = 1
    data[3, 3, 0] = 1
    data[2, 4, 0] = 1

    data[0, 0, 1] = 0.5
    data[1, 1, 1] = 0.5
    data[2, 2, 1] = 0.5
    data[3, 3, 1] = 0.5
    data[2, 4, 1] = 0.5

    print(data)
    print(data.shape)

    return {"out": data}
