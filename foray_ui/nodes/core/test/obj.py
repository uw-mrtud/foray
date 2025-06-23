from foray import port


def config():
    class out:
        inputs = {}
        outputs = {
            "a": {
                "a1": port.Integer,
                "a2": port.Real,
                "a_obj": {"b1": port.ArrayComplex, "b2": port.Dynamic},
            }
        }
        parameters = {}

    return out


def compute(input, parameters):
    out = {"a1": 1, "a2": 1.1}
    return {"a": out}
