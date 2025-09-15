# Node Canvas
This crate implements the node canvas UI.

## State

- camera: location/zoom level
- nodes: list of things that
  - have a position
  - render to the canvas
  - handle click events
    - handle_click(point_in_node_space)->ClickType
    ClickType: Node/InputPort(name)/OutputPort(Name)
- edges: list of ((Node + input_port_name) (Node + output_port_name))
- edge_draw function: given a start and end point, a status (normal, creating, deleting), create a (Path,Stroke)
