# Builder stage
FROM rust:1.71.0 AS builder
WORKDIR /app
RUN apt update && apt install lld clang -y 
ENV SQLX_OFFLINE true
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \ 
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/webapi webapi 
COPY --from=builder /app/configuration.yaml configuration.yaml
ENTRYPOINT ["./webapi"]