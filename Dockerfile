# Frontend build stage
FROM node:18-alpine AS frontend-builder

WORKDIR /frontend
COPY frontend/package*.json ./
RUN npm ci

COPY frontend ./
RUN npm run build

# Rust build stage
FROM rust:1.88-slim AS rust-builder

WORKDIR /app

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests and build dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy source and build application
COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy built application
COPY --from=rust-builder /app/target/release/G3GS ./
COPY --from=rust-builder /app/migrations ./migrations

# Copy frontend build output
COPY --from=frontend-builder /frontend/.next/server/app ./frontend
COPY --from=frontend-builder /frontend/.next/static ./frontend/_next/static
COPY --from=frontend-builder /frontend/public/favicon.ico ./frontend/favicon.ico

# Create non-root user
RUN useradd -r -s /bin/false appuser
RUN chown -R appuser:appuser /app
USER appuser

EXPOSE 3000