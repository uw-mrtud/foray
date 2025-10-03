import numpy as np
from PIL import Image
from pathlib import Path


# create images from csv colormap data
# data from: https://colorcet.com/download/CETperceptual_csv_0_255.zip
# info: https://colorcet.com/index.html
def gen_image(csv_path):
    data = np.genfromtxt(csv_path, delimiter=",", dtype=np.uint8)

    # reshape to Nx1 image
    data = data.reshape((data.shape[0], 1, 3))
    Image.fromarray(data).save(Path(csv_path).with_suffix(".png"))
    data.tofile(Path(csv_path).with_suffix(".bin"))


gen_image("./CET-L20.csv")
gen_image("./CET-C7.csv")
gen_image("./CET-C6.csv")
gen_image("./CET-C3.csv")
