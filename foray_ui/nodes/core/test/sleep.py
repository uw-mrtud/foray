from foray import port
import time


def config():
    class out:
        inputs = {"in": port.Real}
        outputs = {"out": port.Real}
        parameters = {}

    return out


def compute(input, _):
    output = input["in"]

    time.sleep(2)
    print(output)
    return {"out": output}
