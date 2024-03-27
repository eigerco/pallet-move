#!/bin/bash
cd $(dirname $0)
# Estimate needed gas for a single script execution.
smove node rpc estimate-gas-execute-script -s build/multiple-signers/script_transactions/init_module.mvt

