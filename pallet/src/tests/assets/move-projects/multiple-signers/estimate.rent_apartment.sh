#!/bin/bash
cd $(dirname $0)
# Our students who want to rent a dorm together.
ALICE=5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
CHARLIE=5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y
EVE=5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw
# Estimate needed gas for a single script execution.
smove node rpc estimate-gas-execute-script -a $ALICE -s build/multiple-signers/script_transactions/rent_apartment.mvt --cheque-limit 100000000000000

