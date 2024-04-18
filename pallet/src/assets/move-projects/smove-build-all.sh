#!/bin/bash

# Position the cwd in the same folder with the script (where the below folders are located)
cd $(dirname $0)

build_dir=(
    "balance"
    "car-wash-example"
    "get-resource"
    "move-basics"
    "multiple-signers"
    "signer-scripts"
)
bundle_dir=(
    "testing-move-stdlib"
    "testing-substrate-stdlib"
    "using_stdlib_natives"
)

# Build simple packages
for dir in "${build_dir[@]}"; do
    echo $dir
    build_script=$dir"/build.sh"
    if [ -f "$build_script" ];
    then
        sh $build_script
    else
        smove build -p $dir
    fi
done

# Build bundles
for dir in "${bundle_dir[@]}"; do
    echo $dir
    smove bundle -p $dir
done