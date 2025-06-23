import numpy as np
from foray import port, ui


def config():
    class out:
        inputs = {}
        outputs = {"out": port.ArrayReal}  # //3d
        parameters = {
            "Nx": ui.NumberField,
            "Ny": ui.NumberField,
            "RFx": ui.Slider,
            "RFy": ui.Slider,
        }

    return out


def compute(_, parameters):
    b = np.tile(
        np.array([parameters["RFx"], parameters["RFy"]], dtype=np.float64),
        (parameters["Nx"], parameters["Ny"], 1),
    )
    return {"out": b}
