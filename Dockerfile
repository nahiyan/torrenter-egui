# Base stage
FROM rust:1.85-slim AS base
WORKDIR /app
COPY cxx ./cxx
COPY src ./src
COPY build.rs Cargo.lock Cargo.toml ./
RUN apt update && apt install -y git cmake clang libboost-dev libssl-dev libcrypto++-dev

# Development stage
FROM base AS build-dev
RUN cargo build

FROM scratch AS dev
COPY --from=build-dev /app/target/debug/torrenter .

# Production stage
FROM base AS build-prod
RUN cargo build --release

FROM scratch AS prod
COPY --from=build-prod /app/target/release/torrenter .
