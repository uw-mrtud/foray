# Creating Nodes
Nodes consist of two functions: `configure` and `compute`.

TODO: specific code example

## `configure`
The `configure` function returns how the node is structured.

A Node can have 
- inputs
    - the ports on the top of the node
    - these values are passed into the `compute` function
- parameters
    - These values are are also passed into the compute function, but not through ports
    - paramter values are configured for the node in the left-sidebar
- outputs
    - the ports on the bottom of the node
    - these values will be returned from `compute`


## `compute`
The compute function takes input port values and parameter values, does some calculation, and then returns output values


# Project Structure
TODO

# Reloading
TODO

