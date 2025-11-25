# Rust MCP Server

A Model Context Protocol (MCP) server written in Rust that provides AI-assisted code analysis tools for Rust projects. This server enables AI agents to perform advanced code analysis, navigation, and refactoring suggestions on Rust codebases.

## Features

- **File Analysis**: Parse and check Rust files for syntax errors
- **Workspace Indexing**: Build comprehensive call graphs, type usage graphs, and module dependency graphs
- **Code Navigation**: Find definitions and references of symbols
- **Code Smell Detection**: Identify unused functions, long functions, high complexity code, and god objects
- **Refactoring Suggestions**: Automated suggestions for code improvements

## Architecture

The server uses:
- **rmcp** crate for MCP protocol handling
- **syn** for Rust AST parsing
- **tokio** for async operations
- Thread-safe caching with **RwLock**
- Visitor pattern for AST traversal

## Docker Deployment

This project includes Docker support for easy deployment:

- **Multi-stage Dockerfile**: Optimized build process with separate build and runtime stages
- **Nightly Rust**: Uses Rust nightly for compatibility with latest MCP SDK features (edition 2024)
- **Docker Compose**: Simple orchestration for development and production
- **Minimal runtime image**: Based on Debian slim for smaller image size
- **No exposed ports**: MCP communication happens via stdio within containers

## Installation

### Prerequisites

- Docker and Docker Compose
- Git (optional, for cloning the repository)

**Note**: The Docker build uses Rust nightly to support the latest MCP SDK features (edition 2024). No local Rust installation is required.

### Quick Start with Docker

1. **Clone the repository:**
```bash
git clone https://github.com/MythEclipse/rust-mcp-server.git
cd rust-mcp-server
```

2. **Build and run with Docker Compose:**
```bash
docker-compose up --build
```

3. **Or build and run manually:**
```bash
# Build the Docker image
docker build -t rust-mcp-server .

# Run the container
docker run rust-mcp-server
```

### Manual Build (Alternative)

If you prefer to build manually:

#### Prerequisites

- Rust 1.70+ (2024 edition)
- Cargo

#### Build from Source

```bash
git clone https://github.com/MythEclipse/rust-mcp-server.git
cd rust-mcp-server
cargo build --release
```

## Usage

### Running the MCP Server

#### With Docker (Recommended)

```bash
# Quick start with Docker Compose
docker-compose up --build

# Or run directly with Docker
docker run rust-mcp-server

# Run in background
docker run -d rust-mcp-server
```

#### Manual Run (Alternative)

If you built manually:

```bash
# Run the compiled binary
./target/release/rust-mcp-server

# Or run with Cargo
cargo run
```

### MCP Protocol Integration

This server implements the Model Context Protocol (MCP) and communicates using JSON-RPC 2.0 over stdio. It's designed to be integrated with MCP-compatible clients like AI coding assistants.

### Available Tools

#### 1. Check File
Parse and check a Rust file for syntax errors.

**Parameters:**
- `path`: Absolute path to the Rust file to check

**Example MCP Call:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "check_file",
    "arguments": {
      "path": "/path/to/your/file.rs"
    }
  }
}
```

**Example Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "File parsed successfully with no syntax errors."
      }
    ]
  }
}
```

#### 2. Index Workspace
Index all Rust files in a directory and build comprehensive analysis graphs.

**Parameters:**
- `root`: Root directory path to index

**Example MCP Call:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "index_workspace",
    "arguments": {
      "root": "/path/to/rust/project"
    }
  }
}
```

**Returns:**
- Call graph (function relationships)
- Type usage graph (where types are used)
- Module dependency graph
- Function information (complexity, line count, parameters)
- Struct and enum information
- Unused function detection
- Refactoring suggestions

#### 3. Goto Definition
Find the definition location of a symbol.

**Parameters:**
- `name`: Symbol name to find definition for

**Example MCP Call:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "goto_definition",
    "arguments": {
      "name": "MyStruct"
    }
  }
}
```

#### 4. Find References
Find all references to a symbol.

**Parameters:**
- `name`: Symbol name to find references for

**Example MCP Call:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "find_references",
    "arguments": {
      "name": "my_function"
    }
  }
}
```

### Practical Usage Examples

#### Analyzing a Rust Project

1. **Start the MCP server:**
```bash
cargo run
```

2. **Index your workspace** (in another terminal or MCP client):
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "index_workspace",
    "arguments": {
      "root": "/home/user/my-rust-project"
    }
  }
}
```

3. **Check for syntax errors in a specific file:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "check_file",
    "arguments": {
      "path": "/home/user/my-rust-project/src/main.rs"
    }
  }
}
```

4. **Find where a function is defined:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "goto_definition",
    "arguments": {
      "name": "process_data"
    }
  }
}
```

#### Integration with AI Assistants

The server is designed to work with AI coding assistants that support MCP. Here's how it typically integrates:

1. **The AI assistant starts the MCP server** as a subprocess
2. **Communication happens via stdio** using JSON-RPC 2.0
3. **The assistant can call tools** to analyze code, navigate, and get suggestions
4. **Results are used** to provide intelligent code assistance

#### Command Line Testing

You can test the server manually using tools like `socat` or by writing a simple client. For development purposes, you can also use the built-in tests:

```bash
# Run all tests
cargo test

# Run specific tool tests
cargo test test_index_workspace
cargo test test_check_file
```

### Code Analysis Output

When you index a workspace, the server returns comprehensive analysis including:

**Function Analysis:**
- Functions longer than 50 lines
- Functions with complexity > 10
- Functions with > 5 parameters

**Structural Analysis:**
- Structs with > 10 fields
- Enums with > 10 variants

**Dependency Analysis:**
- Functions calling > 10 other functions
- Functions called by > 10 other functions
- Structs used in > 10 places (god objects)

**Unused Code:**
- Private functions that are never called

### Error Handling

The server returns structured errors for invalid requests:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "details": "Path does not exist"
    }
  }
}
```

## Development

### Building

#### With Docker

```bash
# Build the Docker image
docker build -t rust-mcp-server .

# Or use Docker Compose
docker-compose build
```

#### Manual Build

```bash
cargo build
```

### Testing

#### With Docker

```bash
# Run tests in Docker container
docker run --rm rust-mcp-server cargo test
```

#### Manual Testing

```bash
cargo test
```

### Development Workflow

1. **Make changes to the code**
2. **Test locally:**
   ```bash
   cargo test
   ```
3. **Build Docker image:**
   ```bash
   docker build -t rust-mcp-server .
   ```
4. **Test the Docker image:**
   ```bash
   docker run rust-mcp-server
   ```

### Code Analysis Features

The server performs several types of code analysis:

1. **Function Analysis**
   - Line count detection (>50 lines flagged)
   - Cyclomatic complexity calculation (>10 flagged)
   - Parameter count analysis (>5 parameters flagged)

2. **Structural Analysis**
   - Large struct detection (>10 fields)
   - Large enum detection (>10 variants)

3. **Dependency Analysis**
   - Functions calling too many others (>10 callees)
   - Functions called by too many others (>10 callers)
   - God object detection (structs used in >10 places)

4. **Unused Code Detection**
   - Private functions that are never called

## Configuration

### MCP Client Configuration

To use this MCP server with MCP-compatible clients, configure it to run the Docker container:

#### For Claude Desktop (claude_desktop_config.json)

```json
{
  "mcpServers": {
    "rust-mcp-server": {
      "command": "docker",
      "args": ["run", "--rm", "rust-mcp-server"]
    }
  }
}
```

#### For VS Code or other MCP clients

```json
{
  "mcpServers": {
    "rust-mcp-server": {
      "command": "docker",
      "args": ["run", "--rm", "rust-mcp-server"]
    }
  }
}
```

#### Using Docker Compose

If you prefer using Docker Compose:

```json
{
  "mcpServers": {
    "rust-mcp-server": {
      "command": "docker-compose",
      "args": ["exec", "rust-mcp-server", "rust-mcp-server"],
      "cwd": "/path/to/rust-mcp-server"
    }
  }
}
```

### Configuration Parameters

- **`command`**: The executable to run (`docker` or `docker-compose`)
- **`args`**: Array of arguments to pass to the command
- **`cwd`**: Current Working Directory - only needed for docker-compose setup, should point to the directory containing `docker-compose.yml`

### Environment Variables

The server doesn't require any environment variables for basic operation. However, you can set:

- `RUST_LOG`: Set logging level (e.g., `info`, `debug`, `trace`)

### Docker Image Management

```bash
# Build the image
docker build -t rust-mcp-server .

# Run with custom environment variables
docker run -e RUST_LOG=debug rust-mcp-server

# Run with volume mounts (if you need to access host files)
docker run -v /host/path:/container/path rust-mcp-server
```

### Configuration Options

The server uses default configurations and doesn't require additional setup. All analysis is performed on-demand when tools are called.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure `cargo test` passes
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.