import numpy as np
from foray import port


def config():
    return {
        "inputs": {"a": (port.Float, [None, None])},
        "outputs": {"out": (port.Float, [None, None])},
    }


def compute(input, _):
    a = input["a"]
    out = np.fft.fftshift(np.fft.fft2(a)).real
    print(a.shape, a.dtype)
    print(out.shape, out.dtype)

    return {"out": out}
