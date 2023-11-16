#!/bin/bash

# Default values
COVERAGE=false

# Get the root directory of the project
CARGO_TOML_ROOT=$(cargo locate-project --workspace --message-format plain)
CARGO_ROOT_DIR=$(dirname "$CARGO_TOML_ROOT")
export CARGO_TARGET_DIR="$CARGO_ROOT_DIR/target"

# ******************************
# ***** GET THE PARAMETERS *****
# ******************************

# Loop through arguments and process them
for arg in "$@"
do
	case $arg in
		-c|--coverage)
		COVERAGE=true
		shift # Remove argument name from processing
		;;
	esac
done

# ******************************
# ******************************

if $COVERAGE; then
    echo "[COVERAGE] Cleaning last coverage run"
    
    # Clean repository if coverage enabled
    rm -f "$CARGO_ROOT_DIR"/*.profraw
    rm -f "$CARGO_ROOT_DIR"/libafb/*.profraw

    # Clean the latest coverage files
    rm -rf "$CARGO_TARGET_DIR/debug/coverage"

    # Clean the builds in order to be sure to rebuild with the right flag
    cargo clean

    # Export the flags allowing to compile
    export RUSTFLAGS="-Cinstrument-coverage"

    # Export profile file name format
    export LLVM_PROFILE_FILE="binding_test-%p-%m.profraw"
fi


# use libafb development version if any
export LD_LIBRARY_PATH="/usr/local/lib64:$LD_LIBRARY_PATH"
export PATH="/usr/local/lib64:$PATH"
clear

if ! test -f "$CARGO_TARGET_DIR/debug/examples/libafb_demo.so"; then
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

if $COVERAGE; then
    # Create coverage report
    grcov . -s . --binary-path "$CARGO_TARGET_DIR/debug/" -t html --branch --ignore-not-existing -o "$CARGO_TARGET_DIR/debug/coverage/"

    echo "[COVERAGE] Report(s) available under: $CARGO_TARGET_DIR/debug/coverage/"
fi