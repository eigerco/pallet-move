# Move Pallet

A pallet for substrate based blockchains to enable the usage of smart contracts written in the Move-language.

## Overview

Smart contracts can directly be implemented and executed as Move scripts or modularized in Move modules. Therefor, the pallet supports publishing of Move modules and the execution of Move scripts to achieve this functionality. In the case of larger projects, the pallet provides the publishing of a bundle (multiple Move modules).

For the execution of Move scripts and the publication of Move modules, the Move source code has to be compiled and serialized into bytecode. For this purpose, the tool [`smove`](https://github.com/eigerco/smove) is provided. The tool also provides further helpful features for developing and working with the Move language and this pallet.

For more information and to learn more about how to work with this pallet, the following entry points are provided:
- [Move Script Example](#move-script-example)
- [Pallet Configuration in a Substrate-Node](#pallet-configuration-in-a-substrate-node)
- [Quickstart Guide with a Template-Node](#substrate-node-with-move-pallet)
- [Benchmarking](#benchmarking)


## Move Script Example

A basic sample of the Move module and the Move script is shown below.

```move
module DeveloperBob::CarPool {
    // ...

    /// Requests the rent of a car on the given day and between given daytime hours.
    /// Returns `true` in case of success, otherwise `false`.
    public fun rent_car(account: &signer, day: u16, lease_time: u16, token_limit: u128): bool {
        // ...
    }

    // ...
}
```

More details about the module above in [our tutorial](TODO). For this example, the module got published and the following script only needs to be executed.

```move
script {
    use DeveloperBob::CarPool;

    fun rent_car(who: signer, day: u16, lease_time: u16, token_limit: u128) {
        CarPool::rent_car(&signer, day, lease_time, token_limit)
    }
}
```

For a general overview and further details of the Move language, have a look at the [Move-Book](https://move-language.github.io/move/introduction.html).


## Pallet Configuration in a Substrate-Node

The pallet's configuration is very short. Besides the regular `RuntimeEvent` and a predefined `WeightInfo`, you only have to tell the pallet about your `Currency` handler:
```rust
impl pallet_move::Config for Test {
    type Currency = Balances; // here pallet-balances is used
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}
```

As mentioned before, the pallet provides three extrinsic calls for publishing modules and bundles and for the execution of scripts:
```rust
pub fn execute(origin: OriginFor<T>, transaction_bc: Vec<u8>, gas_limit: u64) -> DispatchResultWithPostInfo;
pub fn publish_module(origin: OriginFor<T>, bytecode: Vec<u8>, gas_limit: u64) -> DispatchResultWithPostInfo;
pub fn publish_module_bundle(origin: OriginFor<T>, bundle: Vec<u8>, gas_limit: u64) -> DispatchResultWithPostInfo;
```

Have a look at the [mockup implementation](https://github.com/eigerco/pallet-move/blob/main/tests/mock.rs) for further coding details.


## Quickstart Guide with a Template-Node

### Substrate-node with Move pallet

To spin up the local development node follow instructions from the [official guide](https://docs.substrate.io/tutorials/build-a-blockchain/build-local-blockchain/).

The most important steps are described below.

1. Clone Move pallet and node-template (make sure they are in the same repository) and switch to the newest branch which already include Move pallet building:
```bash
git clone https://github.com/eigerco/pallet-move.git
git clone https://github.com/eigerco/substrate-node-template-move-vm-test
cd substrate-node-template-move-vm-test
git checkout pallet-move
```

2. Create a new branch (if you are going to make new changes) and build the node:
```bash
git checkout -b move-vm-pallet
cargo build --release
```

If you wish to collect runtime benchmarks, you will need to build Move assets first and then add `--features runtime-benchmarks` to the build command. 

Make sure you've Move language installed and accessible from the command line (`move` command). If not, please follow the official [Move tutorial](https://github.com/move-language/move/blob/main/language/documentation/tutorial/README.md) to install it. Then, from the root of the repository, run:

```bash
cd tests/assets/move
move build
cargo build --release --features runtime-benchmarks
```

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

Check if the Move pallet is available in the frontend. If yes, there is possibility to call `execute` extrinsic and observe emitted events. 

To check if there are RPC calls available run:
```bash
curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "mvm_gasToWeight", "params": [123]}' http://localhost:9944/
```
You should see the correct response with some value like:
```bash
{"jsonrpc":"2.0","result":{"ref_time":2123123,"proof_size":0},"id":1}
```

### Docker
There is a possibility to generate the docker image containing a working `node-template` with `move-pallet` built-in. To generate an image, run:
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

It will start the `node-template` on the local interface. You can change the default behavior by passing your own command when running the docker image. All available options are in the [node template](https://docs.substrate.io/reference/command-line-tools/node-template/) documentation.

## Benchmarking
Benchmarking and updating weights should be done each time a new extrinsic is added to the pallet (weights are used to estimate transaction fees). Weights are obligatory for extrinsics that are available for users.

To update weights, simply run:
```bash
./target/release/node-template benchmark pallet --chain dev --pallet pallet-move --steps=50 --repeat=20 --wasm-execution=compiled --output ../pallet-move/src/weights.rs --template ./.maintain/frame-weight-template.hbs --extrinsic '*'
```
when being in the substrate-based node directory root. The assumption is made that the pallet is located under the `../pallet-move` directory. The template for the weights is located under the `./.maintain/frame-weight-template.hbs` directory and can be obtained from the Substrate repository.
