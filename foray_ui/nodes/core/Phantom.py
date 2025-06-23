import numpy as np
from PIL import Image

from foray import node, port


def config():
    return node(
        {},
        {"out": port.ArrayReal},
    )


def compute(input, _):
    img = Image.open("nodes/core/data/slogan_med.png")

    # just take blue channel for simplicity
    np_img = np.array(img)[:, :] / 255.0

    return {"out": np_img}
