from foray import port


def config():
    class out:
        inputs = {"a": port.ArrayReal}
        outputs = {"a1": port.Integer, "a2": port.Real}
        parameters = {}

    return out


def compute(input, parameters):
    out = {"a1": input["a"]["a1"], "a2": input["a"]["a2"]}
    return out
