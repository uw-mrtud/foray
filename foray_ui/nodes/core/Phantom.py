import numpy as np
from PIL import Image

from foray import port


def config():
    return {
        "outputs": {"out": (port.Float, [None, None])},
    }


def compute(_in, _):
    img = Image.open("nodes/core/data/slogan_med.png")

    # just take blue channel for simplicity
    np_img = np.array(img)[:, :] / 255.0
    print("phantom: ", np_img.shape)

    return {"out": np_img}
