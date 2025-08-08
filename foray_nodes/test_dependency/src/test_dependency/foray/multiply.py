import numpy as np
from foray import ForayConfig, Port


def config():
    return (
        ForayConfig()
        .inputs(
            {
                "a": Port.array(Port.float, [None, None]),
                "b": Port.array(Port.float, [None, None]),
            }
        )
        .outputs(
            {
                "out": Port.array(Port.float, [None, None]),
            }
        )
    )


def compute(input, _):
    a = input["a"]
    b = input["b"]
    out = np.multiply(a, b)
    return {"out": out}
