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
        .parameters({"size": NumberField(4), "length": Slider(0.1, 10, 1)})
    )


def compute(_, p):
    N = int(p["size"])
    l = 20

    q = 5
    # Define 3D grid of points
    x = np.linspace(-l, l, N)
    y = np.linspace(-l, l, N)

    z_factor = 2
    z = np.linspace(-l / z_factor, l / z_factor, N // z_factor)
    X, Y, Z = np.meshgrid(x, y, z, indexing="ij")

    p1 = np.asarray([0.0, p["length"], 0.0])
    p2 = np.asarray([0.0, -p["length"], 0.0])

    d1sr = (X - p1[0]) ** 2 + (Y - p1[1]) ** 2 + (Z - p1[2]) ** 2
    d2sr = (X - p2[0]) ** 2 + (Y - p2[1]) ** 2 + (Z - p2[2]) ** 2

    min_dist = 2.4
    f = q / np.maximum(d1sr, min_dist) - q / np.maximum(d2sr, min_dist)

    f_norm = (f - f.min()) / np.ptp(f)
    print(f_norm.min())
    print(f_norm.max())
    print(np.ptp(f_norm))

    # return {"out": f_norm[:, N // 4 : (N // 4) * 3, :]}
    return {"out": f_norm}
