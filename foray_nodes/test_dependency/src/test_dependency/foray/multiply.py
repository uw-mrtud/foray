import numpy as np
from foray import ForayConfig, Port


def config():
    return (
        ForayConfig()
        .inputs(
            {
                "a": Port.float,
                "b": Port.float,
            }
        )
        .outputs(
            {
                "out": Port.float,
            }
        )
    )


def compute(input, _):
    a = input["a"]
    b = input["b"]
    out = a * b
    return {"out": out}
