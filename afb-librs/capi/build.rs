/*
 * Copyright (C) 2015-2022 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use std::process::Command;

fn main() {
    // ============== LIBAFB C interface =====================
    let header = "
        // -----------------------------------------------------------------------
        //         <- private 'libafb' Rust/C unsafe binding ->
        // -----------------------------------------------------------------------
        //   Do not exit this file it will be regenerated automatically by cargo.
        //   Check:
        //     - build.rs at project root for dynamically mapping
        //     - src/capi/libafb-map.h for static values
        // -----------------------------------------------------------------------
        ";
    // probe for dependencies
    #[cfg(not(feature = "rpm_build"))]
    system_deps::Config::new().probe().unwrap();

    // invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=capi/libafb-map.h");

    // expand static inline libafb macro
    let output=Command::new("bash")
        .args(["-c","gcc -E capi/libafb-map.h |sed 's/static *inline//'|sed '/^#/d'| sed '/^$/d' >capi/_libafb-map.c"])
        .output()
        .expect("fail to excec gcc -E capi/_libafb-map.h");
    assert!(output.status.success());

    let _capi_map = bindgen::Builder::default()
        // main entry point for wrapper
        .header("capi/_libafb-map.c")
        .raw_line(header)
        // default wrapper config
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .derive_debug(false)
        .layout_tests(false)
        .allowlist_function("errno")
        .allowlist_function("strerror_r")
        .allowlist_function("__errno_location")
        .allowlist_function("afb_.*")
        .allowlist_type("afb_syslog_.*")
        .allowlist_type("afb_epoll_.*")
        .allowlist_type("afb_req_subcall_flags")
        .allowlist_var("afbBinding.*")
        .blocklist_item("json_object")
        // generate libafb wrapper
        .generate()
        .expect("Unable to generate libafb");

    _capi_map
        .write_to_file("capi/_libafb-map.rs")
        .expect("Couldn't write libafb!");

    // Tell Cargo that if the given file changes, to rerun this build script.
    // println!("cargo:rerun-if-changed=src/capi/libafb-map.c");
    // Use the `cc` crate to build a C file and statically link it.
    cc::Build::new()
        .file("capi/_libafb-map.c")
        .include("/usr/local/include")
        .include("/usr/include/linux")
        .compile("afb-glue");

    // ============== JSONC-C interface =====================
    // add here any special search path specific to your configuration
    println!("cargo:rustc-link-search=/usr/local/lib64");
    println!("cargo:rustc-link-arg=-ljson-c");

    // invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=capi/jsonc_map.h");

    let header = "
    // -----------------------------------------------------------------------
    //         <- private 'json-c' Rust/C unsafe binding ->
    // -----------------------------------------------------------------------
    //   Do not exit this file it will be regenerated automatically by cargo.
    //   Check:
    //     - build.rs at project root for dynamically mapping
    //     - src/capi/jsonc_map.h for static values
    // -----------------------------------------------------------------------
    ";

    let jsonc = bindgen::Builder::default()
        // main entry point for wrapper
        .header("capi/jsonc-map.h")
        .raw_line(header)
        // default wrapper config
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .derive_debug(false)
        .layout_tests(false)
        .allowlist_function("json_object_.*")
        .allowlist_type("json_object_.*")
        .allowlist_function("json_tokener_.*")
        .allowlist_type("json_tokener_error")
        .allowlist_type("lh_table")
        .blocklist_item("printbuf")
        .blocklist_item("json_tokener")
        .blocklist_item("json_tokener_state")
        //.blocklist_item("json_object_array_sort")
        .blocklist_item("json_object_to_json_string_fn")
        .blocklist_item("json_object_.*_userdata")
        .blocklist_item("json_object_set_serializer")
        .blocklist_item("json_tokener_srec")
        .blocklist_item("json_object_delete_fn")
        // generate jsonc wrapper
        .generate()
        .expect("Unable to generate jsonc");

    jsonc
        .write_to_file("capi/_jsonc-map.rs")
        .expect("Couldn't write jsonc!");
}
