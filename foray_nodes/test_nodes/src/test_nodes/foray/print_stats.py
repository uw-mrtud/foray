import numpy as np
from foray import ForayConfig, Port


def config():
    return ForayConfig().inputs(
        {
            "a": Port.array(Port.float, [None, None]),
        }
    )


def compute(input, _):
    a = np.array(input["a"])
    print("Stats: ")
    print(a.shape)
    print("max: ", a.max())
    print("min: ", a.min())
    return {}
