FROM rust:latest as builder
# RUN rustup toolchain install nightly && cargo install cargo-fuzz
COPY . /butter/
WORKDIR /butter
RUN cargo build

FROM debian:bullseye-slim
COPY --from=builder /butter/target/debug/butter /butter/