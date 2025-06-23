from foray import port


def config():
    class out:
        inputs = {"a": port.Dynamic}
        outputs = {"out": port.Dynamic}
        parameters = {}

    return out


def compute(input, _):
    return {"out": input["a"]}
