#!/bin/bash
cd $(dirname $0)
# Participants SS58-addresses.
BOB=5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
ALICE=5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
CHARLIE=5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y
EVE=5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw
# Build the Move sources.
smove build
# Generate script-transactions.
# 1. init_module(Bob)
smove create-transaction --compiled-script-path build/multiple-signers/bytecode_scripts/init_module.mv --args signer:$BOB
# 2. rent_apartment(Alice, Alice-Stash, Bob-Stash)
smove create-transaction --compiled-script-path build/multiple-signers/bytecode_scripts/rent_apartment.mv --args signer:$ALICE signer:$CHARLIE signer:$EVE
