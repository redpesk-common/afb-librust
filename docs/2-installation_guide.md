# Installation

## Install from RPM/APT

WARNING: as today afb-librust remains in early access phase and binary version are not public. Alpha tester should build it from source.

* Declare redpesk repository: [(see doc)]({% chapter_link host-configuration-doc.setup-your-build-host %})

* redpesk: `sudo dnf install afb-librust afb-ui-devtools`
* Fedora: `sudo dnf install afb-librust afb-ui-devtools`
* OpenSuse: `sudo zipper install afb-librust afb-ui-devtools`
* Ubuntu: `sudo apt-get install afb-librust afb-ui-devtools`

## Rebuilding from source

### Rust binding Dependencies

afb-librust require gcc to compile the glue between libafb/C and libafb/Rust. This glue is generated automatically through bindgen
but nevertheless require gcc/C compiler.

* Declare redpesk repository: [(see doc)]({% chapter_link host-configuration-doc.setup-your-build-host %})

* gcc
  * afb-libafb
  * json-c

* rust
  * rustc >1.60
  * cargo >1.60
  * bindgen >0.6

* sample
  * serde-json

Note: bingen & serde pull many other rust modules that are pull automatically by cargo at build time.

### Building

```bash
    git clone git.ovh.iot:redpesk/redpesk-labs/afb-librust.git
    cd afb-librust
    touch libafb/src/capi/build.rs  # cargo fail to detect missing file that should be generated
    cargo build
```
