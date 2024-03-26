#!/bin/bash
cd $(dirname $0)
# Our students who want to rent a dorm together.
BOB=5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
# Estimate needed gas for a single script execution.
smove node rpc estimate-gas-publish-module -a $BOB -m build/multiple-signers/bytecode_modules/Dorm.mv

