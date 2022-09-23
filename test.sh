#!/bin/bash

set -e
# set -x

cargo build

FAILURES=()

for file in $(find ~/rust/wasmrun -name *.wasm); do
    echo $file

    if ! target/debug/parse_wasm $file 2>/dev/null 1>&2; then
        FAILURES+=($file)
    fi
done

echo ''
echo 'Failures:'
echo ''

for failure in $FAILURES; do
    echo $failure
done
