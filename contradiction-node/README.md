# Contradiction Node

### Tech
Uses hyper to expose an API and for node-to-node communications.
Uses a toml file to configure the node itself.

Implements zk circuits using two libraries:
- halo2

### Config File
~~~
[API]
address = String
port = u16

[DB]
path = String
pool_min = Option<u8>
pool_max = Option<u8>
pragma = Option<String>
timeout = Option<u8>
~~~
