# GitHub Actions Setup for Docker Hub Deployment

This document explains how to set up GitHub Actions for automated Docker Hub deployment.

## Prerequisites

1. **Docker Hub Account**: Create an account at [hub.docker.com](https://hub.docker.com)
2. **GitHub Repository**: This workflow should be in the `MythEclipse/rust-mcp-server` repository
3. **Docker Hub Repository**: Create a repository called `rust-mcp-server` under your Docker Hub account

## Setup Instructions

### 1. Create Docker Hub Access Token

1. Go to [Docker Hub](https://hub.docker.com) and sign in
2. Click on your profile → Account Settings → Security
3. Create a new Access Token with "Read, Write, Delete" permissions
4. Copy the generated token

### 2. Add GitHub Secrets

In your GitHub repository, go to **Settings → Secrets and variables → Actions** and add these secrets:

#### Required Secrets:

- **`DOCKERHUB_USERNAME`**: Your Docker Hub username (e.g., `mytheclipse`)
- **`DOCKERHUB_TOKEN`**: The access token you created in step 1

### 3. Verify Repository Access

Make sure the GitHub Actions has permission to push to the `mytheclipse/rust-mcp-server` Docker Hub repository.

### 4. Test the Workflow

1. Push this workflow file to your repository
2. Go to the **Actions** tab in GitHub
3. You should see the workflow running automatically
4. Check the Docker Hub repository to confirm the image was pushed

## Workflow Features

### Automated Triggers

The workflow runs automatically on:
- Push to `main` or `master` branch
- Pull requests to `main` or `master` branch

### Image Tagging Strategy

Images are tagged with:
- `latest` - for the default branch (main/master)
- Branch name - for branch-specific builds
- SHA hash - for unique identification
- PR number - for pull request builds

### Build Optimization

- **Buildx**: Multi-platform builds (currently amd64, can be extended)
- **Caching**: GitHub Actions cache for faster builds
- **Metadata**: Automatic labels and annotations

### Testing

The workflow includes a test job that:
- Pulls the built image
- Verifies the container starts
- Tests basic MCP protocol initialization

## Manual Workflow Dispatch

You can also trigger the workflow manually from the GitHub Actions tab.

## Troubleshooting

### Authentication Issues

If you see authentication errors:
1. Verify `DOCKERHUB_USERNAME` and `DOCKERHUB_TOKEN` are set correctly
2. Make sure the token has "Read, Write, Delete" permissions
3. Check that the Docker Hub repository exists and you have access

### Build Failures

If the build fails:
1. Check the build logs in GitHub Actions
2. Verify the Dockerfile is correct
3. Make sure all dependencies are available

### Push Failures

If push fails:
1. Verify you have write access to `mytheclipse/rust-mcp-server`
2. Check Docker Hub rate limits
3. Ensure the repository name matches exactly

## Security Notes

- **Never commit tokens**: Always use GitHub secrets
- **Token permissions**: Use minimal required permissions
- **Repository access**: Limit to necessary repositories
- **Regular rotation**: Rotate access tokens periodically

## Cost Considerations

- **GitHub Actions**: Free for public repositories, paid minutes for private
- **Docker Hub**: Free for public repositories
- **Build time**: ~3-5 minutes per build
- **Storage**: ~90MB per image

## Advanced Configuration

### Multi-Platform Builds

To add ARM64 support, modify the workflow:

```yaml
- name: Build and push Docker image
  uses: docker/build-push-action@v5
  with:
    context: .
    push: true
    platforms: linux/amd64,linux/arm64
    tags: ${{ steps.meta.outputs.tags }}
    labels: ${{ steps.meta.outputs.labels }}
```

### Scheduled Builds

To add nightly builds:

```yaml
on:
  push:
    branches: [ main, master ]
  pull_request:
    branches: [ main, master ]
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM UTC
```

### Additional Testing

Add more comprehensive tests:

```yaml
- name: Run MCP protocol tests
  run: |
    # More detailed MCP testing
    docker run --rm mytheclipse/rust-mcp-server:latest < test_input.json
```