# Chain Registry API

A REST API for the Cosmos [Chain Registry](https://github.com/cosmos/chain-registry).

## Features

TODO

## TODO for MVP
- [ ] Capture live seeds
- [ ] Capture live peers
- [ ] Raw chain endpoint
- [ ] Raw chain assetlist endpoint
- [ ] Chain index endpoint
- [ ] persistent peer endpoint with filter 
- [ ] seed endpoint with filter
- [ ] capture rpc
- [ ] rpc endpoint with filter
- [ ] capture api/lcd
- [ ] lcd endpoint with filter
- [ ] capture grpc
- [ ] grpc endpoint with filter
- [ ] Save IBC information
- [ ] IBC info endpoint
- [ ] Generate OpenAPI spec
- [ ] Host OpenAPI documentation on Github Pages (setup CICD)

## Future TODO
### High Priority
- [ ] Client command line interface
- [ ] Web frontend

### Nice to Have
- [ ] Rust client library
- [ ] Go client library

## Why Rust?

Selfishly, I wanted to learn Rust. I've been following Rust since 2017, but have never had purpose to write Rust. 
This project is now that purpose. 

Additionally, I wanted something super high performance and robust, so I could host as cheaply as possible.

## To Improve
- [ ] Some postgres text columns should be enums, but I didn't want to toil with getting sqlx and enums to work together.