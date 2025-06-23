import numpy as np
from foray import port, ui

N = 12


def config():
    class out:
        inputs = {}
        outputs = {"out": port.ArrayReal}
        parameters = {
            "X0": ui.NumberField,
            "Y0": ui.NumberField,
            "Z0": ui.NumberField,
            "Nx": ui.NumberField,
            "Ny": ui.NumberField,
            # "t1": ui.Slider,
            # "t2": ui.Slider,
        }

    return out


def compute(_, parameters):
    b = np.tile(
        np.array(
            [float(parameters["X0"]), float(parameters["Y0"]), float(parameters["Z0"])],
            dtype=np.float64,
        ),
        (int(parameters["Nx"]), int(parameters["Ny"]), 1),
    )

    return {
        "out": b,
        # "spin_parameters": {"t1": parameters["t1"], "t2": parameters["t2"]},
    }
