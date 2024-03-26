#!/bin/bash
cd $(dirname $0)
# Our students who want to rent a dorm together.
BOB=5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
# Estimate needed gas for a single script execution.
smove node rpc estimate-gas-execute-script -a $BOB -s build/multiple-signers/script_transactions/init_module.mvt --cheque-limit 100000000000000

