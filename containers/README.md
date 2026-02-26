# Container Deployment

uwu-admin supports multiple container runtimes:

- **[Docker](docker/)** — Docker and Docker Compose setup
- **[Quadlet](quadlet/)** — Podman Quadlet files for running as a systemd service

## Public deployments

The container serves plain HTTP on port 8080 and is designed to sit behind a
TLS-terminating reverse proxy. **Do not expose the container directly to the
internet.** See [Production Deployment](../README.md#production-deployment) in
the main README for details on TLS and recommended environment variables.
