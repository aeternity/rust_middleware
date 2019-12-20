# æternal

## Overview

The middleware is a caching and reporting layer which sits in front of the nodesof the [aeternity blockchain](https://www.aeternity.com). Its purpose is to respond to queries faster than the node can do, and to support queries that for reasons of efficiency the node cannot or will not support itself.

On startup, the middleware reads the entirety of the blockchain, and stores a denormalized version of the data in a PostgreSQL database.

Full documentation is available in the [manual](docs/middleware-guide.md).

## Features

### Caching

The middleware answers a set of queries (listed below) on behalf of the node. Everything it doesn't itself answer it will forward to the node to handle. The goal here is to answer more quickly than the node, without sacrificing correctness.

### Aggregation

A set of queries will list, for example, all transactions for a given account, all which transfer from one user to another, all between certain block heights, and so on

### Contracts

The middleware unpacks and stores all contract calls for which it has the bytecode, with the function called and the arguments. In the near future we will also store the return type and value. You can see all contract calls for a given contract.

### State channels

You can see all active state channels, and for a given state channel, all of the related on-chain transactions

### Names

You can see all names registered, and also which names refer to a given address (reverse lookup).

### Oracles

You can see all active oracles. In the near future we will create an endpoint which lists all questions and responses from an oracle

### Websocket

(see below for more information).

The middleware permits you to subscribe to events via a websocket, and receive updates when the chain state changes. Currently you can subscribe to key blocks, micro blocks, and soon to all events involving a particular on-chain object (contract, account, ...).

## How to use the middleware

### Use ours!

There is a hosted middleware for the æternity mainnet at https://mainnet.aeternal.io/, and one for the testnet at https://testnet.aeternal.io.

### Install your own

- Install a postgresql DB somewhere. Version 11.2 or greater are supported.
- as the postgresql admin user, execute `scripts/prepare-db.sql` which will create the DB and user
- copy 'Rocket.example.toml' to 'Rocket.toml' and edit as you see fit
- copy `.env.example` to `.env`
- if you want to use a different DB name, edit `scripts/prepare-db.sql`, `.env` and `Rocket.toml`

### Run via Docker

This setup runs node, compiler, postgres and aeternal together.

In the `examples` folder there are 2 setup for docker:
- `examples/docker-compose.yml` that includes also the node configuration
- `examples/docker-compose.existing-node.yaml` that relies on an existing node

### Run docker including the node
- Install Docker and Docker Compose.
- Update `examples/node/config.yaml` as per your requirement.
- From the project `examples` folders run `docker-compose up`

### Run docker including relying on an existing node
- Install Docker and Docker Compose
- Make sure you can reach the node external port from your machine
- Edit the file `examples/docker-compose.existing-node.yaml` and set the url of your node as explained in the file
- From the project `examples` folder run `docker-compose up -f docker-compose.existing-node.yaml`

### Tips and tricks

You can run several instances of the middleware simultaneously, with different options. A sensible way of doing this would be one or more using the `-s` option to serve clients, and one (and only one) with the `-p` option, populating the database.

If you don't want to interrupt service, want to update the database with new features, and can live with short-term (possible) inconsistencies, use the `-H` option with the whole chain to force a reload, but serve from the old version soon.

**DON'T USE `diesel migration run`!
**

We now update migrations automatically on application start--and the command `diesel migration run` overwrites important files.

## How to build

You need a nightly rust build

`rustup default nightly`

then

`cargo build`

The middleware will automatically set up its DB on initialization, and run migrations after an update, if they are necessary.

On Ubuntu 18 and 19 the following packages are needed: `libpq-dev`, `libssl-dev` and `zlib1g-dev`. YMMV depending on whether you've e.g. gcc installed.

## How to run

### Development mode

`cargo run -- ` + flags below

### Release mode

```
cargo build --release # make a release build--this will take a long long time
./target/release/mdw # + flags below
```

### Flags
```
        --help        Prints help information
    -H, --heights     Adds or replaces a set of heights, or ranges of heights separated by
    		      commas to the database. i.e. -H1,3-4,6,100-200
    -d, --daemonize   If set, the middleware will daemonize on startup
    -p, --populate    Populate DB
    -s, --server      Start server
    -v, --verify      Check the DB against the chain
    -V, --version     Prints version information

```

## Environment variables

`NODE_URL` - the URL of the æternity node
`AESOPHIA_URL` - if present, the middleware will attempt to use this to decode contract calls, storing the function called, and its parameters
`PID_FILE` - if present, and the `-d` option is set, the middleware stores its pid in this file
`LOG_DIR` - if present, this directory is used for logs, otherwise stdout is used
`DATABASE_URL` - PostgreSQL connection URL
`STATUS_MAX_BLOCK_AGE`

## Supported queries
```
GET /middleware/channels/active
GET /middleware/channels/transactions/address/<address>
GET /middleware/compilers
GET /middleware/contracts/all
GET /middleware/contracts/calls/address/<address>
GET /middleware/contracts/transactions/address/<address>
POST /middleware/contracts/verify
GET /middleware/generations/<from>/<to>?<limit>&<page>
GET /middleware/height/at/<millis_since_epoch>
GET /middleware/names/<name>
GET /middleware/names?<limit>&<page>
GET /middleware/names/active?<limit>&<page>
GET /middleware/names/reverse/<account>?<limit>&<page>
GET /middleware/oracles/list?<limit>&<page>
GET /middleware/oracles/<oracle_id>?<limit>&<page>
GET /middleware/reward/height/<height>
GET /middleware/size/current
GET /middleware/size/height/<height>
GET /middleware/status
GET /middleware/transactions/account/<account>/count
GET /middleware/transactions/account/<sender>/to/<receiver>
GET /middleware/transactions/account/<account>?<limit>&<page>
GET /middleware/transactions/interval/<from>/<to>?<limit>&<page>
GET /middleware/transactions/rate/<from>/<to>

GET /v2/generations/current
GET /v2/generations/height/<height>
GET /v2/key-blocks/current/height
GET /v2/key-blocks/hash/<hash>
GET /v2/key-blocks/height/<height>
GET /v2/micro-blocks/hash/<hash>/header
GET /v2/micro-blocks/hash/<hash>/transactions
GET /v2/micro-blocks/hash/<hash>/transactions/count
GET /v2/transactions/<hash>
```

## Websocket support

    The websocket is exposed by default on 0.0.0.0:3020 but can be overridden with the environment variable `WEBSOCKET_ADDRESS`. On our hosted sites it is available as wss://mainnet.aeternal.io/websocket and wss://testnet.aeternal.io/websocket, which are the recommended URLs.

Message format:
```
{
"op": "<operation to perform>",
"payload": "<message payload>"
}
```
### Supported ops:
- subscribe
- unsubscribe

### Supported payload:
- KeyBlocks
- MicroBlocks
- Transactions
- TxUpdate
- object, which takes a further field, 'target'--see below

### Object subscriptions

You may subscribe to any æternity object type, and be sent all transactions which reference the object. There is an example of this below.

### Returned data

Subscriptions return the array of subscriptions (possibly empty):
```
{"op":"Subscribe", "payload": "KeyBlocks"}
["KeyBlocks"]
{"op":"Subscribe", "payload": "MicroBlocks"}
["KeyBlocks","MicroBlocks"]
{"op":"Unsubscribe", "payload": "MicroBlocks"}
["KeyBlocks"]
{"op":"Subscribe", "payload": "Object", "target": "ak_2eid5UDLCVxNvqL95p9UtHmHQKbiFQahRfoo839DeQuBo8A3Qc"}
["KeyBlocks","MicroBlocks", "ak_nv5B93FPzRHrGNmMdTDfGdd5xGZvep3MVSpJqzcQmMp59bBCv"]

```

Actual chain data is wrapped in a JSON structure identifying the subscription to which it relates:
```
{"payload":{"beneficiary":"ak_nv5B93FPzRHrGNmMdTDfGdd5xGZvep3MVSpJqzcQmMp59bBCv","hash":"kh_rTQLY9ymNL9MSop3RJtySjxPreuowhejHJYu4KSPBfSSZTerh","height":38263,"miner":"ak_bCCis9P7hTGCjfhDPZh3KQUSDXd5V7thyxYAznFcsr1iLn1Uv","nonce":15041902164601590529,"pow":[11019073,17901996,22830222,28569083,49447859,49462425,49519772,87145865,99866878,102237553,123920118,132218818,155018387,180362216,198154991,201321139,222409093,280306327,281351614,283186475,285893935,287852747,323975163,324943621,371752985,397769028,399438380,400871099,414481284,419637507,437183279,446297926,458204825,460525976,468647042,479097364,479884030,491013280,503534491,505981207,522008475,527020581],"prev_hash":"kh_dxbyza4hSY3Ly5U7HJnfn748Po5pJ1rv3dSgisrexcQ5Nna6p","prev_key_hash":"kh_dxbyza4hSY3Ly5U7HJnfn748Po5pJ1rv3dSgisrexcQ5Nna6p","state_hash":"bs_2mnjZYzxN23QpUx7MT2f5RRx8sXcZqvEq7GtqE28LCowq91k4u","target":503824559,"time":1550244690796,"version":1},"subscription":"KeyBlocks"}
```


### Testing the websocket

Here is some magic you can run in your Javascript console
```
var exampleSocket = new WebSocket("ws://127.0.0.1:3020");
exampleSocket.onopen = function (event) {  // when connection is open, send a subscribe request
    exampleSocket.send('{"op":"Subscribe", "payload": "KeyBlocks"}');
    //to unsubscribe: exampleSocket.send('{"op":"Unsubscribe", "payload": "KeyBlocks"}')
}

exampleSocket.onmessage = function (event) {
   	console.log(event.data); // you get data here when it arrives
}

```
