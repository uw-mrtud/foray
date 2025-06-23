# from gpy import PortType
# import numpy as np
import numpy as np
from foray import port, ui


def config():
    class out:
        inputs = {}
        outputs = {"out": port.ArrayReal}
        parameters = {
            "Nx": ui.NumberField,
            "Ny": ui.NumberField,
            "Gx": ui.Slider,
            "Gy": ui.Slider,
        }

    return out


# TODO: pass paramaters as their correct data type, not strings
def compute(_, parameters):
    gx = float(parameters["Gx"])
    gy = float(parameters["Gy"])
    x = np.linspace(0, gx, int(parameters["Nx"]))
    y = np.linspace(0, gy, int(parameters["Ny"]))
    X, Y = np.meshgrid(x, y)

    G = np.zeros_like(X)

    G += X
    G += Y

    return {"out": G}
