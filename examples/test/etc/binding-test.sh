#!/bin/bash

# use libafb development version if any
export LD_LIBRARY_PATH="/usr/local/lib64:$LD_LIBRARY_PATH"
export PATH="/usr/local/lib64:$PATH"
clear

if ! test -f $CARGO_TARGET_DIR/debug/examples/libafb_demo.so; then
    cargo build --example afb_demo
    if test $? != 0; then
        echo "FATAL: fail to compile libafb sample"
        exit 1
    fi
fi

# rebuilt test binding
cargo build  --example afb_test
if test $? != 0; then
    echo "FATAL: fail to compile test suite"
    exit 1
fi

# start binder with test config
afb-binder --config=examples/test/etc/binding-test-auto.json
