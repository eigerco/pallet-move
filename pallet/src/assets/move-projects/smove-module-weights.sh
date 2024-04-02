#!/bin/bash
cd $(dirname $0)
# Our students who want to rent a dorm together.
BOB=5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
# Estimate needed gas for a single script execution.
smove node rpc estimate-gas-publish-module -a $BOB -m move-basics/build/move-basics/bytecode_modules/EmptyBob.mv
smove node rpc estimate-gas-publish-module -a $BOB -m using_stdlib_natives/build/using_stdlib_natives/bytecode_modules/Vector.mv
smove node rpc estimate-gas-publish-module -a $BOB -m car-wash-example/build/car-wash-example/bytecode_modules/CarWash.mv
smove node rpc estimate-gas-publish-module -a $BOB -m multiple-signers/build/multiple-signers/bytecode_modules/Dorm.mv
