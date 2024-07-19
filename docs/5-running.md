# Running/Testing

## Testing

Make sure that your dependencies are reachable from rust scripting engine, before starting your test.

```bash
    afb-binder -vvv --config=afb-samples/etc/binding-config.json
    firefox localhost://1234/devtools
```

## Debugging from vscode/codium

Rust generate standard ELF object and either gdb or lldb permit Rust source debug. Nevertheless in
order gdb/lldb to understand RUST memory structures rust-lldb/rust-gdb should be installed. When installing
with ```rustup``` rust-lldb is installed by default.

On vscode/codium install following extensions

* Rust-analyzer
* CodeLLDB
* Clangd
* Dev-Container (for containerized SDK local-builder)

## Compiling/debugging from a container

check https://github.com/fulup-bzh/mkTinyRootFs

## Configure binder services/options

As rust binding are 100% compatible with C/C++ ones also afb-rust does not require any new option/config.

```bash
# start binder process using a config.json file
afb-binder --config=config.json
```

```json
{
  // binder global options
  "name": "afb-rust",
  "no-ldpaths": true,
  "port": 1234,
  "alias": ["/devtools:/usr/share/afb-ui-devtools/binder"],
  "monitoring": true,
  "binding": [
    // one config per C/C++/Rust binding
    {
        "uid": "rust-api",
        "path": "./target/$HOSTNAME/debug/examples/libafb_demo.so",
        "info": "RUST sample API binding (Rust)"
    }
  ],
}
```
