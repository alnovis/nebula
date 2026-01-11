# Build stage
# Using nightly for edition2024 support (many crates now require it)
FROM rustlang/rust:nightly-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconf

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy src to cache dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "// dummy" > src/lib.rs

# Build dependencies only (this will be cached)
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

# Copy actual source code
COPY src ./src
COPY templates ./templates
COPY migrations ./migrations

# Build the application
RUN cargo build --release

# Runtime stage
FROM alpine:3.19

RUN apk add --no-cache ca-certificates

WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/nebula /app/nebula

# Copy static assets and templates
COPY static ./static
COPY templates ./templates
COPY migrations ./migrations

# Create content directory (will be mounted)
RUN mkdir -p content

# Create non-root user
RUN adduser -D -u 1000 nebula && chown -R nebula:nebula /app
USER nebula

EXPOSE 3000

ENV RUST_LOG=nebula=info,tower_http=info

CMD ["./nebula"]
