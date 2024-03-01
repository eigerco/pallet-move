# Shell Snippets

### Estimate Gas for Module `CarWash`

Requirement: Before publishing the module for user _Bob_.

```sh
smove node rpc estimate-gas-publish-module --account-id 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty --module-path build/car-wash-example/bytecode_modules/CarWash.mv
```

### Estimate Gas for Script `initial_coin_minting`

Requirement: After publishing the module for user _Bob_.
```sh
smove node rpc estimate-gas-execute-script -a 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty -s build/car-wash-example/script_transactions/initial_coin_minting.mvt --cheque-limit 0
```

### Estimate Gas for Script `register_new_user`

Requirement: After publishing the module for an arbitrary user.
```sh
smove node rpc estimate-gas-execute-script -a 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY -s build/car-wash-example/script_transactions/register_new_user.mvt --cheque-limit 0
```

### Estimate Gas for Script `buy_coin`

Requirement: After registering a user with same user. Works for buying one coin.
```sh
smove node rpc estimate-gas-execute-script -a 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY -s build/car-wash-example/script_transactions/buy_coin.mvt --cheque-limit 1000000000000
```

### Estimate Gas for Script `wash_car`

Requirement: unknown.
```sh
smove node rpc estimate-gas-execute-script -a 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY -s build/car-wash-example/script_transactions/wash_car.mvt --cheque-limit 0
```
