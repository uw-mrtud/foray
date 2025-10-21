from enum import StrEnum

type PortType = PrimitivePortType | ArrayType | dict

type ArrayShape = list[int | None]
type ArrayType = tuple[PortType, ArrayShape]


class PrimitivePortType(StrEnum):
    integer = "Integer"
    float = "Float"
    complex = "Complex"
    boolean = "Boolean"
    string = "String"


class Port:
    integer = PrimitivePortType.integer
    float = PrimitivePortType.float
    complex = PrimitivePortType.complex
    boolean = PrimitivePortType.boolean
    string = PrimitivePortType.integer

    @staticmethod
    def array(port_type: PortType, port_shape: ArrayShape):
        return (port_type, port_shape)


type ParameterType = tuple[str, dict]


def Slider(start, stop, num_steps):
    return ("Slider", {"start": start, "stop": stop, "default": num_steps})


def NumberField(default_value: float):
    return ("NumberField", {"default": default_value})


def CheckBox(default_value: bool):
    return ("CheckBox", {"default": default_value})


def TextDisplay(content: str):
    return ("TextDisplay", {"content": content})


def FilePicker():
    return ("FilePicker", {"path": ""})


class ForayConfig(dict):
    def inputs(self, input_ports: dict[str, PortType]):
        self["inputs"] = input_ports
        return self

    def outputs(self, output_ports: dict[str, PortType]):
        self["outputs"] = output_ports
        return self

    def parameters(self, parameters: dict[str, ParameterType]):
        self["parameters"] = parameters
        return self
