# Route Planner

A simple map validator and explorer.

NB: the explorer part is still under development



### Usage
Currently there is no way to configure much apart from port (default to 8001)
```
cargo run  # to run the server
```

```
cargo test  # to run test, this needs the server (cargo run) running in another terminal
```

### logging

logging and tracing are still work in progress so the 

```
->> LISTENING on 127.0.0.1:8001

->> HANDLER      - handler_validate
->> HANDLER      - handler_validate
->> HANDLER      - handler_validate
->> ERROR_RESP   - InvalidNodeInEdge

->> ERROR_RESP   - NodeNeedsMoreDriveways

->> HANDLER      - handler_query
->> HANDLER      - handler_validate
->> HANDLER      - handler_query
->> ERROR_RESP   - NodesNotFound
```


### Idea



```
POST: /validate

Body: structure defined in data.json
- deser using serde
- validation
    - node
        - node xy is always >0
        - (id,pos.x,pos.y) is uniq
    - edge
        - (id,source,sink) is uniq
        - source and sink are valid node.id



    - Each driveway connects only existing nodes
        - Error::InvalidNodeInEdge
    - Each driveway is connected on both ends to an intersection node
        - Error::EdgeMissingSourceOrSink
    - Each intersection node has at least two driveways connected
        - Error::NodeNeedsMoreDriveways
    - Each intersection node should be reachable from any other intersection node
        - Error::DisConnectedNodesFound
        - This requires implmentation of a DFS with VISITED node tracking
            The time complexity seems to linear on nodes and edges


GET /query?start=<node_id>&goal=<node_id>

- validation
    start/goal 
        - should be valid nodes on the map

- Return
    - array of nodes to the path 
    - Error::PathNotFound

- Caveat
    - ther could be more that one correct paths, the algorith is DFS+parent tracking
      so it is greedy and is happy with first path found
    - does not handle cycles very effectively


```

### Goals

Requirements for the module:

- Provide an RESTapi endpoint for layout validation
    - Each driveway connects only existing nodes (**done**)
    - Each driveway is connected on both ends to an intersection node (**done**)
    - Each intersection node has at least two driveways connected  (**done**)
    - Each intersection node should be reachable from any other intersection node (*todo*)
        A part of this is necessary for the "/query" api. Once that is present this can reuse it
- Provide an RESTapi endpoint for route planning
    - Operates on the last valid map passed to the layout validation endpoint (**done**)
    - Consumes two intersection IDs (start, goal) and returns a sequence of node and edge ids (**done**)
        - please check `Caveat` for the "/query" api
    - Return the total distance of travel along with the route (*todo*)
    - Endpoint can serve many requests at once  (**done**)

### Arch

This is a basic MVC style webserver. The server controls the View part (with routes). There are currently 3 routes

- [/ping](./src/main.rs) - handled by `handler_ping`
- [/validate](./src/routes.rs) - handled by `handler_validate`
- [/query](./src/routes.rs) - handeled by `handler_query`


These routes use the [Model and its Controller](./src/model.rs). The models is wrapped using Rust datastructures for 
handle sharing and updating through different threads. There is also minimal locking involved in order to update the model


The "/validate" endpoint could use `Rc<RefCell<Node>>` and `Rc<RefCell<Edge>>` to better represent the layout to validate the map
since there is at anypoint only one thread holding the datastructure but this becomes tricky once some async call get thrown into the 
validation step as the datastruture needs then needs to `Send + Sync`, which leads to `Arc` being the choice.
