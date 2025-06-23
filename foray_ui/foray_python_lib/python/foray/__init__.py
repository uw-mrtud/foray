from dataclasses import dataclass, astuple
from enum import Enum
from typing import Dict, Optional, TypeVar

# re-export rust module, built from `maturin develop`
from ._rust_interface import *


class port(str, Enum):
    Integer = "Integer"
    Real = "Real"
    Complex = "Complex"
    ArrayComplex = "ArrayComplex"
    ArrayReal = "ArrayReal"
    Dynamic = "Dynamic"
    # Object = "Object" just nest dictionary definititions!


class ui:
    Slider = "Slider"
    NumberField = "NumberField"
    CheckBox = "CheckBox"


T = TypeVar("T")


@dataclass
class node:
    inputs: Optional[Dict[str, port]]
    outputs: Optional[Dict[str, port]]
    paramaters: Optional[Dict[str, str]]

    def __init__(
        self,
        inputs: Optional[Dict[str, port]] = {},
        outputs: Optional[Dict[str, port]] = {},
        parameters: Optional[Dict[str, str]] = {},
    ):
        if inputs is None:
            self.inputs = {}
        else:
            self.inputs = inputs

        if outputs is None:
            self.outputs = {}
        else:
            self.outputs = outputs

        if parameters is None:
            self.parameters = {}
        else:
            self.parameters = parameters

    def __iter__(self):
        return iter(astuple(self))

    def __getitem__(self, keys):
        return iter(getattr(self, k) for k in keys)
