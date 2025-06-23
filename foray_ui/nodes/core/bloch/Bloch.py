import numpy as np
from foray import port, ui

# from Spins import N


T1_ms = 40
T2_ms = 10
M0 = 1.0


def config():
    class out:
        inputs = {
            "spins": port.ArrayReal,
            "gradient": port.ArrayReal,
            "rf": port.ArrayReal,
        }
        outputs = {
            "out": port.ArrayReal,
        }
        parameters = {"time steps": ui.Slider}

    return out


def compute(input, parameters):
    m = input["spins"]
    nx, ny, _ = m.shape
    print(nx, ny)

    g = input.get("gradient", np.zeros((nx, ny)))
    print(g)
    rf = input.get("rf", np.zeros((nx, ny, 3)))
    rf = rf / np.sin(rf)
    steps = float(parameters["time steps"])
    steps = int((steps + 1) * 100)

    gamma = 42.577
    usec = 0.001

    rads_per_mT = 2.0 * np.pi * gamma * usec

    fq = 0.1
    db0 = fq / gamma
    mx, my, mz = (
        m[:, :, 0],
        m[:, :, 1],
        m[:, :, 2],
    )

    for i in range(0, steps):
        expt1 = np.exp(-(i * usec / T1_ms))
        # TODO: crusher here?
        expt2 = np.exp(-(i * usec / T2_ms))
        mz0 = M0 * (1.0 - expt1)

        phase = np.zeros_like(m)
        phase[:, :, :] = -rf
        phase[:, :, 2] = g + db0
        phase = phase * rads_per_mT

        mx, my, mz = rotate(
            mx,
            my,
            mz,
            phase[:, :, 0],
            phase[:, :, 1],
            phase[:, :, 2],
        )
        mx = mx * expt2
        my = my * expt2
        mz = mz0 + expt1 * mz

    m_out = np.zeros_like(m)
    m_out[:, :, 0] = mx
    m_out[:, :, 1] = my
    m_out[:, :, 2] = mz
    return {"out": m_out}


def rotate(rx, ry, rz, px, py, pz):
    ######################
    # R = (rx, ry, rz) = (mx, my, mz)
    # P = (px, py, pz)
    # Rotate R about P, by |P| radians
    #
    # R1 is the projection of R on P, which does not rotate
    # R2 is the residue of R which is perpendicular to P
    # R3 and R4 are the vectors of the rotated R2 which are
    #           parallel and perpendicular to R2, respectively
    #
    # R1 = P (P dot R)/(|P|**2)
    # R2 = R - R1
    # R3 = R2 Cos(|P|)
    # R4 = (P x R2) Sin(|P|) / |P|
    #
    # R' = R1+R3+R4

    psqu = px * px + py * py + pz * pz
    if (psqu > 0).any():
        pmag = np.sqrt(psqu)

        # Need to mask, to avoid div_by_0
        pzero = np.zeros(psqu.shape)

        pdotr = px * rx + py * ry + pz * rz
        r1norm = np.where(psqu, pdotr / psqu, pzero)
        cp = np.cos(pmag)
        spn = np.where(pmag, np.sin(pmag) / pmag, pzero)

        r1x = px * r1norm
        r1y = py * r1norm
        r1z = pz * r1norm

        r2x = rx - r1x
        r2y = ry - r1y
        r2z = rz - r1z

        r3x = r2x * cp
        r3y = r2y * cp
        r3z = r2z * cp

        r4x = (py * r2z - pz * r2y) * spn
        r4y = (pz * r2x - px * r2z) * spn
        r4z = (px * r2y - py * r2x) * spn

        rx = r1x + r3x + r4x
        ry = r1y + r3y + r4y
        rz = r1z + r3z + r4z

    return rx, ry, rz
