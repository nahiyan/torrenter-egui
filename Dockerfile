# Base stage
FROM rust:1.85-slim AS base
WORKDIR /app
COPY cxx ./cxx
COPY src ./src
COPY build.rs bindings.rs Cargo.lock Cargo.toml ./
RUN apt update && apt install -y git cmake clang libboost-dev libssl-dev libcrypto++-dev

# Development stage
FROM base AS dev
RUN cargo build

# Production stage
FROM base AS prod
RUN cargo build --release
