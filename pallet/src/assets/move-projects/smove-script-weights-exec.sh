#!/bin/bash
# Note: Modules CarWash (car-wash-example) and Dorm (multiple-signers) have to be published first!
cd $(dirname $0)
# Estimate needed gas for a single script execution.
smove node rpc estimate-gas-execute-script -s car-wash-example/build/car-wash-example/script_transactions/register_new_user.mvt
smove node rpc estimate-gas-execute-script -s car-wash-example/build/car-wash-example/script_transactions/buy_coin.mvt
smove node rpc estimate-gas-execute-script -s car-wash-example/build/car-wash-example/script_transactions/wash_car.mvt
smove node rpc estimate-gas-execute-script -s multiple-signers/build/multiple-signers/script_transactions/rent_apartment.mvt
