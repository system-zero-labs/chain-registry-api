# Chain Registry API

A REST API for the Cosmos [Chain Registry](https://github.com/cosmos/chain-registry).

## Features

TODO

## TODO for MVP

- [x] Capture live seeds
- [x] Capture live peers
- [x] Raw chain endpoint
- [x] Raw chain assetlist endpoint
- [x] Chain index endpoint
- [x] persistent peer endpoint with filter
- [x] seed endpoint with filter
- [ ] capture rpc
- [ ] rpc endpoint with filter
- [ ] capture api/lcd
- [ ] lcd endpoint with filter
- [ ] capture grpc
- [ ] grpc endpoint with filter
- [ ] IBC info endpoint - IBC has a JSONSchema but no chain has used it yet as of 04/2023.
- [ ] Generate OpenAPI spec
- [ ] Host OpenAPI documentation on Github Pages (setup CICD)

## Future TODO

### High Priority

- [ ] Client command line interface
- [ ] Better web frontend

## Why Rust?

Selfishly, I wanted to learn Rust. I've been following Rust since 2017, but have never had purpose to write Rust.
This project is now that purpose.

Additionally, I wanted something super high performance and robust, so I could host as cheaply as possible.

## What if/when the Chain Registry is deprecated and lives on chain?

I still plan to support this API. Instead of caching data from the Chain Registry Github, the API will query chains to
build cached data.
Therefore, you can still have a central API to query for chain information.

There will likely be a long transition period as information moves on-chain and away from Github.

## To Improve

- [ ] Some postgres text columns should be enums, but I didn't want to toil with getting sqlx and enums to work
  together.