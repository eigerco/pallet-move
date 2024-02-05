# #!/bin/bash

# Position the cwd in the same folder with the script (where the below folders are located)
cd $(dirname $0)

build_dir=(
    "balance"
    "car-wash-example"
    "get-resource"
    "move-basics"
    "signer-scripts"
)
bundle_dir=("using_stdlib_natives")

# Build simple packages
for i in "${build_dir[@]}"; do
    echo $i
    rm -rf "$i/build"
done

# Build bundles
for i in "${bundle_dir[@]}"; do
    echo $i
    rm -rf "$i/build"
done
