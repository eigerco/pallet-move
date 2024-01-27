# #!/bin/bash

# Position the cwd in the same folder with the script (where the below folders are located)
cd $(dirname $0)

build_dir=(
    "balance"
    "car-wash-example",
    "get-resource"
    "move-basics"
)
bundle_dir=("using_stdlib_natives")

# Build simple packages
for i in "${build_dir[@]}"; do
    echo $i
    smove build -p $i
done

# Build bundles
for i in "${bundle_dir[@]}"; do
    echo $i
    smove bundle -p $i
done
