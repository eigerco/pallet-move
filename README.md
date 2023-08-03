# pallet-move
MoveVM pallet for Substrate-based chains


# Running pallet from the substrate-node

## Substrate-node with Move pallet
To spin up the local development node follow instructions from the [official guide](https://docs.substrate.io/tutorials/build-a-blockchain/build-local-blockchain/).

The most important steps are described below.

1. Clone Move pallet and node-template (make sure they are in the same repository) and switch to the newest branch which already include Move pallet building:
```bash
git clone https://github.com/eigerco/pallet-move.git
git clone https://github.com/eigerco/substrate-node-template-move-vm-test
cd substrate-node-template-move-vm-test
git checkout polkadot-1.0.0-pallet-move
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

## Benchmarking
Benchmarking and updating weights should be done each time new extrinsic is added to the pallet (weights are used to estimate transaction fees). Weights are obligatory for extrinsics that are available for users.

To update weights simply run:
```bash
./target/release/node-template benchmark pallet --chain dev --pallet pallet-move --steps=50 --repeat=20 --wasm-execution=compiled --output ../pallet-move/src/weights.rs --template ./.maintain/frame-weight-template.hbs --extrinsic '*'
```
when being in the substrate-based node directory root. Assumption is made that the pallet is located under `../pallet-move` directory. Template for the weights is located under `./.maintain/frame-weight-template.hbs` directory and can be obtained from Substrate repository.
