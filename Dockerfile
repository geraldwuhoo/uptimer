# chef
FROM docker.io/library/rust:1.78.0 AS chef
RUN rustup target add x86_64-unknown-linux-musl && \
    apt-get update && \
    apt-get install -y --no-install-recommends musl-tools=1.2.3-1 musl-dev=1.2.3-1 && \
    rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef
WORKDIR /usr/src

# planner
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Builder
FROM chef AS builder
COPY --from=planner /usr/src/recipe.json recipe.json

RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin uptimers

# Clean image
FROM scratch
COPY --from=builder /usr/src/target/x86_64-unknown-linux-musl/release/uptimers /usr/bin/uptimers
COPY --from=builder /usr/lib/ssl/ /usr/local/ssl/
COPY --from=builder /etc/ssl/ /etc/ssl/
ENTRYPOINT ["uptimers"]
