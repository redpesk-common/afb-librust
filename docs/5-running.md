# Running/Testing

## Testing

Make sure that your dependencies are reachable from rust scripting engine, before starting your test.

```bash
    afb-binder -vvv --config=examples/demo/etc/binding-config.json
    firefox localhost://1234
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

Redpesk SDK is traditionally ship as a prebuilt rootfs container. Nevertheless it is possible to rebuilt it from scratch. As here after with podman.

```bash
    # install podman
    dnf/zypper install -y podman

    # optionally define your container storage filesystem
    edit $HOME/.config/containers/storage.conf
      driver    = "btrfs"
      runroot   = "/containers/runroot"
      graphroot = "/containers/graphroot"

    # download your container rootfs
      podman search almalinux
      podman pull docker.io/almalinux/8-base

    # start a container exposing tcp:1234 and map $HOME/Workspace within container namespace
      podman run --name 'my-pod' -dt --cap-add=SYS_PTRACE -p 1234:1234 -v $HOME/Workspace:/home/Workspace:Z docker.io/almalinux/8-base

    # check container and enter container
      podman ps -a ;# list all container event not running one
      podman exec -it 'my-pod' bash  ;# attach from container ID/name
      podman stop 'my-pod'
      podman start 'my-pod'
      podman rm 'my-pod'

    # install rust within container
      dnf install rust cargo clang

    # declare redpesk SDK repository
      cat >/etc/yum.repos.d/redpesk.repo <<EOF
        [redpesk-sdk]
        name=redpesk-sdk
        enabled=1
        autorefresh=0
        baseurl=https://download.redpesk.bzh/redpesk-lts/arz-1.0-update/packages/middleware/x86_64/os/
        type=rpm-md
        gpgcheck=0
EOF

    # install libafb and afb-binder dependencies
      dnf install json-c-devel afb-libafb-devel afb-binder afb-ui-devtools

    # save newly created image and use it to create new containers
      podman stop 'my-pod'
      podman commit my-pod 'redpesk-rust'
      podman run --name 'redpesk-rust' --cap-add=SYS_PTRACE  -p 1234:1234  -dt -v $HOME/Workspace:/home/Workspace:Z localhost/my-image
```

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
  "set" : {
    // one optional config by API
    "rust-api": {
      "key": "any json for binding 'config' control callback"
    }
  }
}
```
