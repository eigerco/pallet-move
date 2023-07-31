# pallet-move
MoveVM pallet for Substrate-based chains


# Running pallet from the substrate-node

## Substrate-node
To spin up the local development node follow instructions from the [official guide](https://docs.substrate.io/tutorials/build-a-blockchain/build-local-blockchain/).

The most important steps are described below.

1. Clone node-template and switch to the newest branch:
```bash
git clone https://github.com/eigerco/substrate-node-template
cd substrate-node-template
git checkout polkadot-v1.0.0
```

2. Create a new branch and build the node:
```bash
git checkout -b move-vm-pallet
cargo build --release
```
If you wish to collect runtime benchmarks, add `--features runtime-benchmarks` to the build command.

3. Run the node:
```bash
./target/release/node-template --dev
```

4. Install and run the frontend (demands `node` and `yarn` to be installed):
```bash
git clone https://github.com/substrate-developer-hub/substrate-front-end-template
cd substrate-front-end-template
yarn install
yarn start
```

5. Your browser should be opened automatically at `http://localhost:8000/` and you should see the information about the node.

Alternatively, there is a possibility to use the Polkadot frontend:
```bash
git clone https://github.com/polkadot-js/apps
cd apps
yarn install
yarn start
```
and open `http://localhost:3000/` in your browser. Then switch on the left-side to the local development chain. Now you can do the same things as with the substrate frontend.

Congratulations! You've spun up your first Substrate node. You are able now to perform your first transaction. To do this, log into the frontend (the node must be up, assuming that you are using the substrate frontend) and: 
1. choose one of the accounts on the right top (eg. Alice account); 
2. look at the `Transfer` section where you can choose the destination account for your transfer. Choose one eg. `charlie`; 
3. enter the amount to be sent; 
4. press `Submit` button to begin the transaction; 
5. when everything is completed you will see information that the transaction has been successful along with `tx hash` value;
6. `Balances` section should now display new balances for the affected accounts.

## Adding MoveVM pallet to the node

1. Go to the substrate-based node directory root.

2. Open the `runtime/Cargo.toml` file and add a new dependency:
```toml
pallet-move = { default-features = false, path = "../../pallet-move" }
```
and under the `[features]` section, to the `std` section, add this feature:
```toml
std = [
    <...>
    "pallet-move/std",
]
```
to the `runtime-benchmarks` section, add this feature:
```toml
runtime-benchmarks = [
    <...>
    "pallet-move/runtime-benchmarks",
]
```
and to the `try-runtime = [` section, add this feature.
```toml
try-runtime = [
    <...>
    "pallet-move/try-runtime",
]
```
The instructions here assume that the `pallet-move` and the `substrate-node-template` repos are located under the same directory.

3. Open `runtime/src/lib.rs` and add new pallet to the runtime configuration.
Add new import:
```rust
pub use pallet-move;
```
Find section where pallets are configured for Runtime and add new pallet:
```rust
impl pallet_move::Config for Runtime {
        type RuntimeEvent = RuntimeEvent;
        type WeightInfo = pallet_move::weights::SubstrateWeight<Runtime>;
}
```
Add new pallet under `construct_runtime!(` macro:
```rust
    MoveModule: pallet_move,
```
If you need to run runtime benchmarks find `define_benchmarks!(` macro and add:
```rust
    [pallet_move, MoveModule]
```

4. Re-build the node, run the node and the frontend, then check if the pallet is available in the frontend. If yes, there is possibility to call `execute` extrinsic and observe emitted events.

## Benchmarking
Benchmarking and updating weights should be done each time new extrinsic is added to the pallet (weights are used to estimate transaction fees). Weights are obligatory for extrinsics that are available for users.

To update weights simply run:
```bash
./target/release/node-template benchmark pallet --chain dev --pallet pallet-move --steps=50 --repeat=20 --execution=wasm --wasm-execution=compiled --output ../pallet-move/src/weights.rs --template ./.maintain/frame-weight-template.hbs --extrinsic '*'
```
when being in the substrate-based node directory root. Assumption is made that the pallet is located under `../pallet-move` directory. Template for the weights is located under `./.maintain/frame-weight-template.hbs` directory and can be obtained from Substrate repository.
