def config():
    return {
        "inputs": {
            "a": ("Float", [None, 3]),
            "b": ("Float", [None, 3]),
        },
        "outputs": {
            "out": ("Float", [None, 3]),
        },
        "parameters": {},
    }


def compute(input, _):
    print(
        {
            "low": {
                "r": 0.1,
                "g": 0.2,
                "b": 0.3,
            }
        }
    )
    return {
        "out": {
            "low": input["color"],
            "high": input["color"],
        }
    }
