FROM lukemathwalker/cargo-chef:latest-rust-1.64.0 as chef
WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached. 
COPY . .

ENV SQLX_OFFLINE=true
RUN cargo build --release --bin z2p

FROM debian:bullseye-slim as runtime
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/z2p z2p
COPY configuration configuration
ENV APP_ENVIRONMENT=production  
EXPOSE 8000
ENTRYPOINT ["./z2p"]