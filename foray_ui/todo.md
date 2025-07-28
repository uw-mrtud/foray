## System Level
- [x] undo/redo

- [x] save to file
  - [x] load from file

- [x] hot reload nodes
  - [x] notify of node errors
    - [*] Actionable Error Messages. Not cryptic! Ideally point directly to what needs to change.

## External input
- [x] Nodes that load data from file
  - [ ] run from gpi "headless" from cli
  - [*] Load data from cli arguments (How others will use your network. Shouldn't have to edit the network to point to new files)

## Python interop
 
- [x] create user configuration file

- [x] load nodes from user configured location
- [ ] Add nodes written from other users via pip install

- [ ] solidfy how the user's python environment will work
 - [ ] make environments reproducible by default. It should always be simple to share work with others
 - [ ] understand how python package management works, well enough to do this properly

- [-] Specific and actionable errors for python nodes
  - [x] Config errors which cause the node to never compute
  - [ ] Runtime Compute errors which will retry compute

- [ ] rename GPI to foray everywhere
- [ ] create `foray` python module
- [ ] use foray python module from pyo3

- [ ] print statements from a node are viewable from the ui 

### Declarative UI
- [x] define parameters/widgets from python
  - [ ] get auto reloading working well

  - [ ] pass data as the expected type (not as str)
  - [ ] Make order of ports match declaration order

  - [ ] specify default values
  - [ ] customize widgets (start, stop,step for sliders, etc.)

## UI
 - [x] pan
   - [ ] kinetic pan
 - [ ] zoom
 - [x] hotkeys
  - [x] delete node
  - [x] deselect node
  - [x] copy/paste
  - [x] add node
 - [x] duplicate node on command + click
 - [ ] toggle auto reload
 - [ ] visually notify node reloads

 - [x] Add node UI with user defined hierarchy (assumed from file stucture?)

- [x] render nodes

- [x] select single node
  - [x] select multiple nodes

## Data Model
- [x] execution
  - [x] async execution
  - [x] parallel execution
  - [ ] pause execution
  - [ ] consistent styling for execution state
    - [x] running indicication (vary alpha over time?)
    - [ ] wire fireing indication
          - after node completion, output wire exponential decay of brightness down to base level
    - [ ] unfilled inputs
      - [ ] allow for optional node inputs

- [x] load available nodes
- [x] display available nodes
- [x] create nodes


- [x] render node types differently
- [x] render nodes ontop of one another in the order that they are most recently selected (Current selected node will always be on top)

- [x] wires
  - [x] create wires via click and drag
  - [x] indicate wires that will be deleted when a new wire replaces an old wire  
- [x] multiple inputs/outputs
  - [x] render input/output types differently
  - [x] semantic color for data type 
  - [x] Text display of allowed array shape
    - [ ] Text display of current array shape
    - [ ] semantic shape for array shape/dimension

- [ ] restrict node connections to only valid ports
  - [?] and convert arrays of data on wires

- [x] display editable node config
  - [x] Specify config UI from python

- [ ] 


## On Canvas Ad-Hoc Visualization 
- [x] efficient image display

- [ ] resizable nodes

- [ ] Visualization methods for different node output types
  - [ ] Output type -> available visualization methods mapping
  - [ ] Show visualization method by default when availble
  - [ ] UI to selecect from available methods
  - [ ] Plot node auto-detects what type of plot to use.

- [ ] image display manipulation
  - [ ] floor window level contrast
  - [ ] complex phase vis

## C interface
- [ ] compilation process


## Primary Visualization/Output
- [?] compose widgets from multiple nodes together

# Bugs
- [ ] Node running while it's deleted, results come back, but node is gone. crash on unwrapping node
- [ ] Node's are selectable when behind left panel
- [x] node src errors cause crash on unwrap 
- [ ] support nodes that need to create an image handle
  - "node update" method that gets called after compute?

- [ ] Nodes save/load absolute/relative position improperly
  - [ ] relative path is not saved

- [ ] Loading a network from UI differs from load on start. Load on start dosn't crash if there's an error with nodes, Load from UI does crash.

# In progress
- [x] Port colors 
- [x] Port type names 

- [ ] Visualization Methods enum
  - [ ] render method
  - [ ] icon method
- [ ] Ports -> Visualiztion mapping 
- [ ] Node data model render cache
- [ ] Trigger cache-clear/re-render on compute complete
- [ ] Node widget toggle
