from foray import CheckBox, ForayConfig, NumberField, Slider, TextDisplay


def config():
    return ForayConfig().parameters(
        {
            "number field": NumberField(4),
            "slider": Slider(0.1, 10, 1),
            "checkbox": CheckBox(True),
            "text": TextDisplay("hello world!!"),
        }
    )


def compute(_v, _p):
    return {}
