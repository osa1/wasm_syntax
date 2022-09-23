#!/bin/bash

set -e
set -x

(cd crates/wasm_syntax; cargo expand --lib > /tmp/wasm_syntax_expanded.rs)

cargo-expand-tidy /tmp/wasm_syntax_expanded.rs > crates/wasm_syntax_expanded/src/lib.rs

rustfmt crates/wasm_syntax_expanded/src/lib.rs
