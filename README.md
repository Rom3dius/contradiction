# contradiction

Currently leaving this in the state its in, the improvements listed below would be awesome to implement,
but now that I have a better understanding of Rust and ZK probably not worth the time at the moment.

## What?
Executes Risc0 zk-circuits, saving the receipt and distributing it to other nodes for verification.
In the default configuration, it acts as a executor, verifier and storage node.
Currently no consensus is implemented.

## What's missing
Currently does not submit proofs to other nodes / doesn't have a verify endpoint.
Ideally a feature to use a different ZK library.
A feature to make a node purely an executor, and another to make a node only a storage/verifier.

## Technical Details and Improvements
The most important crates used in this project are SQLx, anyhow and hyper. Improvements that could be made are the
following:
- Create custom error types and handle those instead of using .downcast_ref() to blanket handle errors.
- Use query! instead of query so queries are checked at compile-time.
- Drop the reqwests crate and create a hyper http client.
- Put node/risc0/executor.rs and node/risc0/models.rs in the contradiction-risc0-methods crate.
- Return a UUID for the receipt to the user immediately and add work to a queue instead of immediately executing.
- Change the node online detection code to utilize the last_ping_at field and implement a grace period.
- Use cargo-nextest for end-to-end testing.
- Use a github action to enforce branch protection and build a release/docker image if the commit is tagged.

## Important Endpoints
### (POST) /api/do-compute (JSON)
Request:
```
{'<circuit name>': {<dictionary of circuit parameters>}}
```
Response:
```
{'status_code': 201, 'text': '<uuid>'}
```
### (GET) /api/fetch-compute (JSON)
Request:
```
{'<circuit name>': {<dictionary of circuit parameters>}}
```
Response:
```
{'status_code': 200, 'text': '<receipt>'}
```

## Less Important Endpoints
 - (POST) /register_node
 - (GET) /ping
 - (GET) /nodes