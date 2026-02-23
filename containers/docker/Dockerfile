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

# Create non-root user and data directory
RUN groupadd -r uwu && useradd -r -g uwu -s /usr/sbin/nologin uwu && \
    mkdir -p /data && \
    chown uwu:uwu /data && \
    chown -R uwu:uwu /var/lib/nginx /var/log/nginx /usr/share/nginx/html

# Copy API binary
COPY --from=backend /build/api/target/release/uwu-admin-api /usr/local/bin/uwu-admin-api

# Copy frontend build to nginx html dir
COPY --from=frontend /build/web/dist/ /usr/share/nginx/html/

# Copy nginx config and entrypoint
COPY containers/docker/nginx.conf /etc/nginx/nginx.conf
COPY containers/docker/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh && \
    chown -R uwu:uwu /usr/share/nginx/html

USER uwu

EXPOSE 8080
VOLUME /data

ENTRYPOINT ["/entrypoint.sh"]
