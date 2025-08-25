# MCP Containerd

This is an MCP server implemented using the RMCP (Rust Model Context Protocol) library for operating Containerd.

## Features

- Implements an MCP server using the RMCP library
- Supports all Containerd CRI interface operations
- Provides Runtime Service interfaces
- Provides Image Service interfaces
- Supports ctr interface

## Prerequisites

- Rust development environment
- Containerd installed and running
- Protobuf compilation tools

## Building

```bash
cargo build --release
```

## Running

```bash
mcp-containerd -t http  #use streamhttp
mcp-containerd --help #list the help info
```

By default, the service will connect to the `unix:///run/containerd/containerd.sock` endpoint.

## Using with simple-chat-client

The simple-chat-client allows you to interact with the MCP Containerd service:
simple-chat-client has moved to [simple-chat-client](https://github.com/modelcontextprotocol/rust-sdk/tree/main/examples/simple-chat-client)


Example interaction:

```
> please give me a list of containers
AI: Listing containers...
Tool: list_containers
Result: {"containers":[...]}

> please give me a list of images
AI: Here are the images in your containerd:
Tool: list_images
Result: {"images":[...]}
```

## Service Structure

The MCP server includes the following main components:

- `version` service: Provides CRI version information
- `runtime` service: Provides container and Pod runtime operations
- `image` service: Provides container image operations

## CRI Interfaces

### Runtime Service

- Create/Stop/Delete Pod Sandbox
- Create/Start/Stop/Delete containers
- Query Pod/container status
- Execute commands in containers

### Image Service

- List images
- Get image status
- Pull images
- Delete images
- Get image filesystem information


## License

Apache-2.0 