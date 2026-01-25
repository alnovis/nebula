# Build stage
FROM rust:1.85-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconf

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Pin incompatible crates to older versions
RUN cargo update home --precise 0.5.9

# Build dependencies only
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY src ./src
COPY templates ./templates
COPY migrations ./migrations

# Touch main.rs to invalidate the dummy build
RUN touch src/main.rs

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
COPY content ./content

# Create non-root user
RUN adduser -D -u 1000 nebula
USER nebula

EXPOSE 3000

ENV RUST_LOG=nebula=info,tower_http=info

CMD ["./nebula"]
