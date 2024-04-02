#!/bin/bash
# Note: Modules CarWash (car-wash-example) and Dorm (multiple-signers) have to be published first!
cd $(dirname $0)
# Estimate needed gas for a single script execution.
smove node rpc estimate-gas-execute-script -s car-wash-example/build/car-wash-example/script_transactions/initial_coin_minting.mvt
smove node rpc estimate-gas-execute-script -s multiple-signers/build/multiple-signers/script_transactions/init_module.mvt
