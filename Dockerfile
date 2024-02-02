# syntax=docker/dockerfile:1
FROM rust:latest as builder
RUN apt-get update && apt-get upgrade -y && apt-get install -y protobuf-compiler make gcc g++ curl clang
WORKDIR /usr/src/move
RUN git clone https://github.com/eigerco/substrate-move.git 
RUN git clone https://github.com/eigerco/pallet-move.git
RUN git clone https://github.com/eigerco/substrate-node-template-move-vm-test.git --branch pallet-move
RUN ./substrate-move/scripts/dev_setup.sh -bypt && cargo install --git https://github.com/eigerco/smove
RUN cd substrate-node-template-move-vm-test && cargo b -r --features runtime-benchmarks

FROM debian:bullseye-slim
RUN apt-get update && apt-get upgrade -y && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/move/substrate-node-template-move-vm-test/target/release/node-template /usr/local/bin/node-template
EXPOSE 9333 9944 30333
CMD ["node-template", "--dev"]
