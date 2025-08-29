# Creating Nodes
Nodes consist of two functions: `configure` and `compute`.

Here is how a simple "multiply" node is defined. This node has two inputs named "a" and "b", that each expect a 2d array of complex values. It multiplies these two values and returns the result.
```python
import numpy as np
from foray import ForayConfig, Port


def config():
    return (
        ForayConfig()
        .inputs(
            {
                "a": Port.array(Port.complex, [None, None]),
                "b": Port.array(Port.complex, [None, None]),
            }
        )
        .outputs(
            {
                "out": Port.array(Port.complex, [None, None]),
            }
        )
    )


def compute(input, _):
    a = input["a"]
    b = input["b"]
    out = np.multiply(a, b)
    return {"out": out}
```

## `configure`
The `configure` function returns how the node is structured.

`ForayConfig()` creates a default node with no ports or parameters.

Calling the `inputs` method lets us specify the input ports of the node.
`inputs` expects key-value pairs, where the keys will be the names of the ports, and the values will be the type of data the port expects.

`outputs` works similarly, defining the names and types of the output ports.

## `compute`
Our second function handles the actual computation of the node. `input` contains the values for each of the input ports. These values are then used to compute the output value, and a key-value pair is returned to populate the output ports.

# Parameters
A node can optionally have additional parameters that can be manipulated graphically in the foray window. See `circle_mask.py` node for an example.
<!---->
<!-- A Node can have  -->
<!-- - inputs -->
<!--     - the ports on the top of the node -->
<!--     - these values are passed into the `compute` function -->
<!-- - parameters -->
<!--     - These values are are also passed into the compute function, but not through ports -->
<!--     - paramter values are configured for the node in the left-sidebar -->
<!-- - outputs -->
<!--     - the ports on the bottom of the node -->
<!--     - these values will be returned from `compute` -->


