FROM node:20-alpine AS frontend
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

FROM rust:1.78-bookworm AS backend
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY --from=frontend /app/frontend/dist ./frontend/dist
RUN cargo build --release -p cirnotorrent-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    p7zip-full \
    unrar \
    openssh-client \
    && rm -rf /var/lib/apt/lists/*

COPY --from=backend /app/target/release/cirnotorrent-server /usr/local/bin/cirnotorrent-server

RUN mkdir -p /data/downloads /data/config /data/watch

ENV WEBUI_PORT=8080 \
    USERNAME=admin \
    PASSWORD="" \
    MAX_DOWN_SPEED=0 \
    MAX_UP_SPEED=0 \
    DOWNLOAD_PATH=/data/downloads

EXPOSE 8080 6881 6881/udp

VOLUME ["/data/downloads", "/data/config", "/data/watch"]

ENTRYPOINT ["cirnotorrent-server"]
CMD ["--port", "8080", "--download-path", "/data/downloads", "--db-path", "/data/config/cirnotorrent.db"]
