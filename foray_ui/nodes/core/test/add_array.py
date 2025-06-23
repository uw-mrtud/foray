# from gpy import PortType
# import numpy as np


def config():
    class out:
        inputs = {"a": "Real", "b": "Real"}
        outputs = {"out": "Real"}

    return out


def compute(input):
    a = input["a"]
    b = input["b"]
    return a + b
