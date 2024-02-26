# Move Pallet Tech Guide

In this tech guide, you will find instructions and details on:
- [Quickstart Guide for the Template-Node](#quickstart-guide-for-the-template-node)
- [Testing](#testing)
- [Pallet Configuration in a Substrate-Node](#pallet-configuration-in-a-substrate-node)
- [Benchmarking](#benchmarking)
- [Docker](#docker)


## Quickstart Guide for the Template-Node

Compile the template-node and launch locally by:

1. Clone Move pallet and node-template (make sure they are in the same directory):
   ```bash
   git clone https://github.com/eigerco/pallet-move.git
   git clone https://github.com/eigerco/substrate-node-template-move-vm-test --branch pallet-move
   ```

2. Build the node:
   ```bash
   cd substrate-node-template-move-vm-test
   cargo b -r
   ```

3. Run the node in developer mode:
   ```bash
   ./target/release/node-template --dev
   ```

For connecting to the template-node via browser using polkadot.js there are two options:
- Use polkadot.js online with your favorite browser to connect to your local running template-node by opening the URL:
  ```
  https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer
  ```
  Therefore, __port 9944 needs to be opened__ by your firewall!

- Alternatively, it can be installed on your computer and be executed locally:
  ```bash
  git clone https://github.com/polkadot-js/apps
  cd apps
  yarn install
  yarn start
  ```
  and open `http://localhost:3000/` in your browser. 
  Then, switch on the left side to the local development chain.
  Now, you can do the same things as with the substrate front-end.


## Testing

Verify that everything works fine by running the pallet's unit tests with all features enabled:
```bash
cargo test --verbose --features build-move-projects-for-test
```

You can also verify that Rust's formatting tool `cargo fmt` was applied by executing:
```bash
cargo fmt --all -- --check
```

Verification for Rust's linter `clippy` by executing:
```bash
cargo clippy --all-targets -- -D warnings
```

_Info: All the above commands are executed in the CI workflow._

If you want to run more MoveVM specific tests, check out [testing strategy in substrate-move](https://github.com/eigerco/substrate-move?tab=readme-ov-file#testing).

Read detailed info about test concepts (what tests cover and how) in the [old testing guide](./milestone-deliverables/m1-testing-guide.md).


## Pallet Configuration in a Substrate-Node

The pallet's configuration is concise. Besides the regular `RuntimeEvent` and a predefined `WeightInfo`, you only have to tell the pallet about your `Currency` handler:
```rust
impl pallet_move::Config for Test {
    type Currency = Balances; // here pallet-balances is used
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = (); // or use pallet_move::weights::SubstrateWeight<Test>;
}
```

The pallet provides three extrinsic calls. To find about those check the [design document](final-design.md).

Have a look at the [mockup implementation](https://github.com/eigerco/pallet-move/blob/main/tests/mock.rs) for further coding details, or check the crate's Rust documentation:
```bash
cargo doc --open
```


## Benchmarking

To enable benchmarking, you have to compile the template-node with the feature `runtime-benchmarks` enabled:
```bash
cargo b -r --features runtime-benchmarks
```

Benchmarking and updating weights should be done each time a new extrinsic is added to the pallet (weights are used to estimate transaction fees). 
Weights are obligatory for extrinsics that are available for users.

To update weights, run:
```bash
./target/release/node-template benchmark pallet \
    --chain dev \
    --wasm-execution=compiled \
    --pallet "pallet-move" \
    --extrinsic "*" \
    --steps 50 \
    --repeat 20 \
    --output weights.rs
```
when being in the substrate-based node directory root (template-node). 
The assumption is made that the pallet is located under the `../pallet-move` directory. 
The template for the weights is located under the `./.maintain/frame-weight-template.hbs` directory and can be obtained from the Substrate repository.


## Docker

There is a possibility of generating the docker image containing a working node-template with move-pallet built-in.
To generate an image, run in the __directory of pallet-move__:
```bash
sudo docker build -t "nodemove:Dockerfile" .
```

When the build is ready, you can check if you can see the image in the docker repository by running:
```bash
sudo docker images
```

To run the image, enter:
```bash
sudo docker run nodemove:Dockerfile
```
It will start the `node-template` within a local Docker container. In dependency how you want to use that container, it needs to get adjusted.

> [!TIP]
> To be able to connect via an external polkadot.js instance to the Docker container, the simplest way is to connect the container to your host's network by running:
> ```bash
> docker run --net host nodemove:Dockerfile
> ```
