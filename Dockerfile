# syntax=docker/dockerfile:1
FROM rust:latest as builder
RUN apt-get update && apt-get upgrade -y && apt-get install -y protobuf-compiler make gcc g++ curl clang
WORKDIR /usr/src/move
RUN git clone https://github.com/move-language/move.git && cd move && ./scripts/dev_setup.sh -bypt && cargo install --path language/tools/move-cli
WORKDIR /usr/src/move/pallet-move
COPY . .
WORKDIR /usr/src/move
RUN cd pallet-move && cd tests/assets/move && move build
RUN git clone https://github.com/eigerco/substrate-node-template && cd substrate-node-template && git checkout polkadot-1.0.0-pallet-move && cargo build --release --features runtime-benchmarks

FROM debian:bullseye-slim
RUN apt-get update && apt-get upgrade -y && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/move/substrate-node-template/target/release/node-template /usr/local/bin/node-template
EXPOSE 9930 9333 9944 30333 30334
CMD ["node-template", "--dev"]
