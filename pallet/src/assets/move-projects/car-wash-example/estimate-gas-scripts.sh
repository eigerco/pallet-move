#! /usr/bin/sh
cd $(dirname $0)
# Estimate needed gas
smove node rpc estimate-gas-execute-script -s build/multiple-signers/script_transactions/rent_apartment.mvt
