# Docker Deployment Guide

This guide covers how to deploy and use the Rust MCP Server using Docker.

## Quick Start

### Using Docker Hub (Recommended)

```bash
# Pull the latest image
docker pull mytheclipse/rust-mcp-server:latest

# Run the MCP server
docker run --rm -i mytheclipse/rust-mcp-server:latest
```

### Using Docker Compose

```bash
# Start the service (uses Docker Hub image automatically)
docker-compose up -d

# View logs
docker-compose logs -f rust-mcp-server

# Stop the service
docker-compose down
```

For development with local changes, uncomment the `rust-mcp-server-dev` service in `docker-compose.yml`.

## Development Setup

### Building Locally

```bash
# Build the Docker image locally
docker build -t rust-mcp-server:local .

# Run the local build
docker run --rm -i rust-mcp-server:local
```

### Development with Docker Compose

The `docker-compose.yml` includes:
- Hot reload for development
- Volume mounting for source code
- Environment-specific configurations

```yaml
version: '3.8'
services:
  mcp-server:
    build: .
    volumes:
      - .:/app
      - /app/target
    environment:
      - RUST_LOG=debug
    command: cargo run
```

## Production Deployment

### Docker Run

```bash
# Run in background
docker run -d --name mcp-server mytheclipse/rust-mcp-server:latest

# Check status
docker ps | grep mcp-server

# View logs
docker logs mcp-server

# Stop and remove
docker stop mcp-server && docker rm mcp-server
```

### Docker Compose (Production)

```yaml
version: '3.8'
services:
  mcp-server:
    image: mytheclipse/rust-mcp-server:latest
    restart: unless-stopped
    environment:
      - RUST_LOG=info
    # Add any necessary volumes or networks
```

## MCP Protocol Usage

The MCP server communicates via stdio using JSON-RPC 2.0 protocol.

### Basic Interaction

```bash
# Send initialize request
echo '{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {
      "name": "example-client",
      "version": "1.0.0"
    }
  }
}' | docker run --rm -i mytheclipse/rust-mcp-server:latest
```

### Available Tools

The server provides these MCP tools:
- `index_workspace`: Analyze workspace for code quality issues
- `check_file`: Check individual files for problems
- `find_references`: Find symbol references
- `goto_definition`: Navigate to symbol definitions

## Configuration

### Environment Variables

- `RUST_LOG`: Set logging level (error, warn, info, debug, trace)

### Volumes

For persistent caching or workspace analysis:
```bash
docker run -v $(pwd):/workspace -i mytheclipse/rust-mcp-server:latest
```

## Troubleshooting

### Common Issues

1. **Container exits immediately**
   - Check logs: `docker logs <container-id>`
   - Verify MCP client is sending proper JSON-RPC messages

2. **Permission denied**
   - Ensure binary has execute permissions
   - Check Docker user permissions

3. **High memory usage**
   - The server caches AST data in memory
   - Restart container to clear cache

4. **Protocol errors**
   - Verify JSON-RPC 2.0 format
   - Check protocol version compatibility

### Debug Mode

```bash
# Run with debug logging
docker run --rm -e RUST_LOG=debug -i mytheclipse/rust-mcp-server:latest

# Interactive debugging
docker run --rm -it mytheclipse/rust-mcp-server:latest /bin/bash
```

## Performance Optimization

### Resource Limits

```bash
docker run \
  --memory=512m \
  --cpus=1.0 \
  --rm -i mytheclipse/rust-mcp-server:latest
```

### Multi-stage Builds

The Dockerfile uses multi-stage builds to minimize image size:
- Builder stage: Full Rust toolchain (~2GB)
- Runtime stage: Minimal Debian with binary (~90MB)

## Security Considerations

- Run as non-root user (future enhancement)
- Use read-only root filesystem where possible
- Regularly update base images
- Scan for vulnerabilities: `docker scan mytheclipse/rust-mcp-server:latest`

## Monitoring

### Health Checks

```bash
# Basic health check
docker run --rm mytheclipse/rust-mcp-server:latest /usr/local/bin/rust-mcp-server --version
```

### Logs

```bash
# Follow logs
docker logs -f mcp-server

# Export logs
docker logs mcp-server > mcp-server.log 2>&1
```

## Integration Examples

### With VS Code Extension

```json
{
  "mcpServers": {
    "rust-analyzer": {
      "command": "docker",
      "args": ["run", "--rm", "-i", "mytheclipse/rust-mcp-server:latest"]
    }
  }
}
```

### With Claude Desktop

```json
{
  "mcpServers": {
    "rust-mcp-server": {
      "command": "docker",
      "args": ["run", "--rm", "-i", "mytheclipse/rust-mcp-server:latest"]
    }
  }
}
```

## Contributing

When contributing:
1. Test your changes with Docker
2. Update this documentation if needed
3. Ensure CI/CD pipeline passes

## Support

- Issues: [GitHub Issues](https://github.com/MythEclipse/rust-mcp-server/issues)
- Documentation: [README.md](README.md)
- Docker Hub: [mytheclipse/rust-mcp-server](https://hub.docker.com/r/mytheclipse/rust-mcp-server)