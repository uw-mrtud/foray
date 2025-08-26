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

- [x] use foray python module from pyo3
- [ ] create `foray` python module
- [ ] publish to pypi

- [ ] print statements from a node are viewable from the ui 

### Declarative UI
- [x] define parameters/widgets from python
  - [x] get auto reloading working well

  - [ ] pass data as the expected type (not as str)
  - [ ] Make order of ports match declaration order

  - [x] specify default values
  - [x] customize widgets (start, stop,step for sliders, etc.)
  - [ ] Define list of all widget types
  - [ ] implement all widget types

- [ ] Add helper functions for defining node input/output/parameter
  - [ ] Possible to use node functions definition directly?? This would make calling the node compute function from python much nicer.

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

- [x] Port colors 
- [x] Port type names 

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

- [ ] Create either a less strict port type option, or make some nodes (like a multiplication node) have generic inputs that get inferred when they are hooked up.
- [ ] restrict node connections to only valid ports
  - [?] and convert arrays of data on wires

- [x] display editable node config
  - [x] Specify config UI from python



## On Canvas Ad-Hoc Visualization 
- [x] efficient image display

- [ ] resizable nodes

- [ ] Visualization methods for different node output types
  - [ ] Output type -> available visualization methods mapping
  - [x] Show visualization method by default when availble
  - [ ] UI to selecect from available methods
  - [x] Plot node auto-detects what type of plot to use.

- [ ] image display manipulation
  - [ ] floor window level contrast
  - [ ] complex phase vis

- [ ] Visualization Methods enum
  - [ ] render method
  - [ ] icon method
- [ ] Ports -> Visualiztion mapping 
- [ ] Node data model render cache
- [ ] Trigger cache-clear/re-render on compute complete
- [ ] Node widget toggle

## C interface
- [ ] test c interface works well

## Primary Visualization/Output
- [?] compose widgets from multiple nodes together

# Bugs
- [ ] Node running while it's deleted, results come back, but node is gone. crash on unwrapping node
- [ ] Node's are selectable when behind left panel
- [x] node src errors cause crash on unwrap 
- [x] support nodes that need to create an image handle

- [x] Nodes save/load absolute/relative position improperly
  - [x] relative path is not saved

- [ ] Loading a network from UI differs from load on start. Load on start dosn't crash if there's an error with nodes, Load from UI does crash.
- [ ] Visualization of transposed arrays doesn't take into account the changed strides

- [ ] unwrap on Err foray_py/src/discover.rs:65:14, No module found
  - probably because I moved the location of the venv. need to re-create the venv
  - handle the error better, to hopefully point the user to what went wrong clearly

- [ ] undo causes crash
  - thread 'main' panicked at foray_ui/src/app.rs:554:49:
      Node should not be idle here!

# Before alpha release
- [x] Python helper API
  - [x] input, output, parameters functions
  - [x] port types
  - [x] parameter types
  - [x] test as a module

- [x] parse available nodes from python runtime
  - [x] iterate over modules, finding any modules with a `foray` submodule
  - [x] account for submodules of `foray`
  - [x] add paths to list of folders that need to be watched for changes
  - [x] call node functions (compute, config) from python runtime environment directly, rather than from src file
  - [x] test that hot-reloading works well
  - [x] node tree display should not show "foray" directories

- [x] publish pypi `foray` module

- [x] parameter types aren't always strings

- [x] complex data type
  - [x] complex visualization (for now just display magnitude)

- [x] headless mode

- [x] design/implement how a virtual environment is chosen.
  - [x] network path as cli argument
  - [x] relative to the network location?
  - [x] network load/save defaults to network folder in same directory as venv

- [ ] minimal documentation on installation, writing nodes, etc.
  - [x] repository with example layout
  - [ ] example nodes
  - [ ] explain port types
  - [ ] explain controls


## Functional Testing
- [ ] Installation
- error handling 
    - [ ] config errors
    - [ ] compute errors
    - [ ] compute return type errors
    - [ ] filter out any invalid nodes
- hot reloading
    - [ ] changes to virtual env (e.g. adding/removing dependencies) should trigger reload


## Minor usability tweaks
- [ ] don't re-execute node on click
- [ ] visualize nodes firing
- [x] async save/load
- [ ] gracefully handle when venv cannot be determined

## Refactor changes
- [ ] put all relevant ui into workspace module
- [ ] reintroduce debug and palette toggles/display
- [x] reimplement subscriptions
