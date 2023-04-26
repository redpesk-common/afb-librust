# Testing your api

Demo provides two samples configuration

* binding-test-api.json: wait for request on http://localhost:1234/devtools
* binding-test;auto.json: start/stop automatically and print tap output on stdout

Technically Test-API is just an other binding, in default testing mode both bindings are loaded
within the same binding and security context (which translate in no privileges verification).

To launch a test a remote target, while keeping your test binder on your host you should:

* export the API on remote target
* import the api on your local testing machine

From default config comment binding lines and add --ws-client binder option to replace demo-binding with a shadow api.

```bash
  "name": "afb-test",
  "alias": ["/devtools:/usr/share/afb-ui-devtools/binder"],
  "ws-client": [
    "tcp:10.18.127.177:9999/rust-api"
  ],

  "binding": [
    // {
    //     "uid": "rust-demo-binding",
    //     "path": "-/home/fulup/.cargo/build/debug/examples/libafb_demo.so",
    //     "info": "RUST sample API binding (Rust)"
    // },
    {
        "uid": "tap-test-binding",
        "path": "/home/fulup/.cargo/build/debug/examples/libafb_test.so",
        "info": "Rust testing sample binding API",
    }
  ],
```

on remote binder add --ws-server to afb-binder

* --ws-server => export an API to external world
* tcp:*:9999 => export through as TCP websocket on port 9999 listening all host network interfaces
* api-rust => the api to export

```bash
afb-binder --ws-server='tcp:*:9999/api-rust' --config=examples/demo/etc/binding-config.json
```

Note:

* ws-server require one port per API. Should a binder export more than one api to an external interface, you should declare --ws-server as often as needed.
* ws-server may also be set within binder config.json file. In this case as for ws-client it should be declared as a json array.
