import numpy as np
from foray import ForayConfig, Port


def config():
    return ForayConfig().outputs(
        {
            "out": Port.float,
        }
    )


def compute(input, _):
    return {"out": 7.0}
