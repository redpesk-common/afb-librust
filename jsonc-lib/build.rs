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
extern crate bindgen;
extern crate cc;

fn main() {
    // add here any special search path specific to your configuration
    println!("cargo:rustc-link-search=/usr/local/lib64");
    println!("cargo:rustc-link-arg=-ljson-c");

    // invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=src/capi/jsonc_map.h");

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
        .header("src/capi/jsonc-map.h")
        .raw_line(header)
        // default wrapper config
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .derive_debug(false)
        .layout_tests(false)
        .allowlist_function("json_object_.*")
        .allowlist_function("json_tokener_.*")
        .allowlist_type("json_tokener_error")
        .allowlist_type("lh_table")
        .blocklist_item("printbuf")
        .blocklist_item("json_tokener")
        .blocklist_item("json_tokener_state")
        .blocklist_item("json_object_array_sort")
        .blocklist_item("json_object_to_json_string_fn")
        .blocklist_item("json_object_.*_userdata")
        .blocklist_item("json_object_set_serializer")
        .blocklist_item("json_tokener_srec")
        .blocklist_item("json_object_delete_fn")
        // generate jsonc wrapper
        .generate()
        .expect("Unable to generate jsonc");

    jsonc
        .write_to_file("src/capi/jsonc-map.rs")
        .expect("Couldn't write jsonc!");
}
