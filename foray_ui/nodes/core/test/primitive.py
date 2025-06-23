from foray import port


class primitive:
    inputs = {}
    outputs = {"a": port.Integer}
    parameters = {}


def config():
    return primitive


def compute(input, parameters):
    return {"a": 1}
