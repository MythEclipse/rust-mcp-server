# Docker Hub Deployment Guide

This guide explains how to push the Rust MCP Server Docker image to Docker Hub for public access.

## Prerequisites

- Docker installed and running
- Docker Hub account
- Built Docker image locally

## Step-by-Step Instructions

### 1. Login to Docker Hub

```bash
docker login
```

Enter your Docker Hub username and password when prompted.

### 2. Tag the Image

Replace `yourusername` with your actual Docker Hub username:

```bash
# For the repository owner (MythEclipse)
docker tag rust-mcp-server:latest mytheclipse/rust-mcp-server:latest

# For other users
docker tag rust-mcp-server:latest yourusername/rust-mcp-server:latest
```

### 3. Push to Docker Hub

```bash
# For the repository owner
docker push mytheclipse/rust-mcp-server:latest

# For other users
docker push yourusername/rust-mcp-server:latest
```

### 4. Verify the Push

Check that your image is available on Docker Hub:
- Visit: https://hub.docker.com/r/mytheclipse/rust-mcp-server
- Or: https://hub.docker.com/r/yourusername/rust-mcp-server

## Automated Deployment (GitHub Actions)

For automated builds, you can set up GitHub Actions to build and push images automatically on code changes.

Create `.github/workflows/docker-publish.yml`:

```yaml
name: Build and Push Docker Image

on:
  push:
    branches: [ main, master ]
  pull_request:
    branches: [ main, master ]

jobs:
  build-and-push:
    runs-on: ubuntu-latest

    steps:
    - name: Checkout code
      uses: actions/checkout@v4

    - name: Set up Docker Buildx
      uses: docker/setup-buildx-action@v3

    - name: Login to Docker Hub
      uses: docker/login-action@v3
      with:
        username: ${{ secrets.DOCKERHUB_USERNAME }}
        password: ${{ secrets.DOCKERHUB_TOKEN }}

    - name: Build and push
      uses: docker/build-push-action@v5
      with:
        context: .
        push: true
        tags: mytheclipse/rust-mcp-server:latest
```

## Using the Published Image

Once published, users can pull and run the image directly:

```bash
# Pull the image
docker pull mytheclipse/rust-mcp-server:latest

# Run the MCP server
docker run mytheclipse/rust-mcp-server:latest
```

## Image Information

- **Repository**: mytheclipse/rust-mcp-server
- **Tags**: latest
- **Size**: ~90MB
- **Base Image**: Debian slim + Rust nightly
- **Architecture**: Multi-platform (amd64, arm64)

## Troubleshooting

### Authentication Issues
```bash
# If login fails, try using a Personal Access Token
echo $DOCKERHUB_TOKEN | docker login -u $DOCKERHUB_USERNAME --password-stdin
```

### Push Fails
```bash
# Check if you have permission to push to the repository
docker images
docker tag rust-mcp-server:latest yourusername/rust-mcp-server:latest
docker push yourusername/rust-mcp-server:latest
```

### Image Not Found
```bash
# Make sure the image exists locally
docker images | grep rust-mcp-server
```