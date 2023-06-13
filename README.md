# Chain Registry API

A REST API for the Cosmos [Chain Registry](https://github.com/cosmos/chain-registry).

## Features

See the [official marketing page](https://chainregistry.xyz).

## Roadmap

In no particular order:

- [x] Liveness for peers
- [ ] Add node id to peer endpoints, so user does not have to parse it from address.
- [ ] Liveness for RPC endpoints
- [ ] Liveness for LCD endpoints
- [ ] Liveness for grpc endpoints
- [ ] Capture uptime metrics for peers and endpoints
- [ ] Capture data such as earliest block height for endpoints

## Ideas

These may or may not happen.

- [ ] Client command line interface
- [ ] Discover and track peers outside the chain registry
- [ ] Discover and track endpoints outside the chain registry 

# FAQ

## Why not use the Chain Registry itself or explorers?

I wanted something programmatic that I could query easily. Digging through some web UI or a Github repo isn't my idea of
fun.
An API lets you write scripts and programs to do the digging for you.

## What if/when the Chain Registry repo is deprecated and lives on chain?

I still plan to support this API. Instead of caching data from the Chain Registry Github, the API will query chains to
build the cache. Therefore, you can still have a central API to query for chain information.

There will likely be a long transition period as information moves on-chain and away from Github.

## Why Rust?

Selfishly, I wanted to learn Rust. I've been following Rust since 2017, but have never had purpose to write any.
This project is now that purpose.

Additionally, I wanted something super high performance and robust, so I could host as cheaply as possible.