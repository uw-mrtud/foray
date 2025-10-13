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
        .parameters({"size": NumberField(10), "fwhm": Slider(0.1, 100, 1)})
    )


def compute(_, p):
    n = int(p["size"])
    fwhm = p["fwhm"]

    x = np.arange(0, n, 1, float)
    y = x[:, np.newaxis]

    x0 = (n - 1) / 2
    y0 = x0

    out = np.exp(-4 * np.log(2) * ((x - x0) ** 2 + (y - y0) ** 2) / fwhm**2)
    return {"out": out}
