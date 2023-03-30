## Compile:

```
 cargo build
 ```

## Test

```
cargo test jsonc
```

## Documentation

```
cargo doc --open
```

## Test coverage
```
# you need Rust grcov
cargo install grcov
rustup component add llvm-tools-preview

CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test jsonc

# generate HTML report
mkdir -p target/coverage/html
grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/html
firefox target/coverage/html/inde.html

# generate LCOV report
mkdir -p target/coverage
grcov . --binary-path ./target/debug/deps/ -s . -t lcov --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/lcov.info

# clean up intermediary files
rm *.profraw

```