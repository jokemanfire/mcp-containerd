# MCP Containerd

This is an MCP server implemented using the RMCP (Rust Model Context Protocol) library for operating Containerd's CRI interfaces.

## Features

- Implements an MCP server using the RMCP library
- Supports all Containerd CRI interface operations
- Provides Runtime Service interfaces
- Provides Image Service interfaces

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
cargo run --release
```

By default, the service will connect to the `unix:///run/containerd/containerd.sock` endpoint.

## Using with simple_chat

The simple_chat client allows you to interact with the MCP Containerd service:

```bash
# First, start the MCP Containerd service
cargo run --release

# In another terminal, run the simple_chat client
cd simple-chat-client
cargo run --bin simple_chat
```

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

## Configuration

Currently using default configuration. Future versions will support customizing connection parameters through configuration files.

## License

Apache-2.0 