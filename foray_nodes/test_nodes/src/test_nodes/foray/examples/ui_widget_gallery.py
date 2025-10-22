import os
from foray import CheckBox, ForayConfig, NumberField, Slider, TextDisplay, FilePicker


def config():
    return ForayConfig().parameters(
        {
            "number field": NumberField(4),
            "slider": Slider(0.1, 10, 1),
            "checkbox": CheckBox(True),
            "text display": TextDisplay("hello world!!"),
            "file": FilePicker(),
        }
    )


def compute(_v, p):
    print(os.getcwd())
    print(p)
    return {}
