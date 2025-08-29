import numpy as np
from foray import ForayConfig, Port


def config():
    return (
        ForayConfig()
        .inputs(
            {
                "a": Port.array(Port.complex, [None, None]),
            }
        )
        .outputs(
            {
                "out": Port.array(Port.complex, [None, None]),
            }
        )
    )


def compute(input, _):
    a = input["a"]
    out = np.fft.fftshift(np.fft.fft2(a))
    return {"out": out}
