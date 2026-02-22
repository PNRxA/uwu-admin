# Stage 1: Build frontend
FROM node:22-alpine AS frontend
WORKDIR /build/web
COPY web/package.json web/package-lock.json* ./
RUN npm ci
COPY web/ ./
RUN npm run build

# Stage 2: Build API
FROM rust:1-slim-bookworm AS backend
WORKDIR /build/api
COPY api/ ./
RUN cargo build --release

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends nginx ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create data directory for SQLite
RUN mkdir -p /data

# Copy API binary
COPY --from=backend /build/api/target/release/uwu-admin-api /usr/local/bin/uwu-admin-api

# Copy frontend build to nginx html dir
COPY --from=frontend /build/web/dist/ /usr/share/nginx/html/

# Copy nginx config and entrypoint
COPY docker/nginx.conf /etc/nginx/nginx.conf
COPY docker/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

EXPOSE 80
VOLUME /data

ENTRYPOINT ["/entrypoint.sh"]
