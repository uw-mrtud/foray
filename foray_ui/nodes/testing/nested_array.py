def config():
    return {
        "inputs": {
            "a": (
                {
                    "color": {
                        "r": "Float",
                        "g": "Float",
                        "b": "Float",
                    }
                },
                [None],
            ),
        },
        "outputs": {},
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
