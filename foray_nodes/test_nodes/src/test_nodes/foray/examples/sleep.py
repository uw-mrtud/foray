import numpy as np
from foray import ForayConfig, NumberField, Port, Slider
import time


def config():
    return (
        ForayConfig()
        .inputs(
            {
                "in": Port.array(Port.float, [None, None]),
            }
        )
        .outputs(
            {
                "out": Port.array(Port.float, [None, None]),
            }
        )
    )


def compute(inputs, p):
    out = inputs.get("in")
    if out is None:
        out = False

    time.sleep(4.0)
    return {"out": out}
