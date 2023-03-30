/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk samples code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 */

#![doc(
    html_logo_url = "https://iot.bzh/images/defaults/company/512-479-max-transp.png",
    html_favicon_url = "https://iot.bzh/images/defaults/favicon.ico"
)]

extern crate jsonc;
extern crate libafb;
extern crate serde;

// import libafb dependencies
libafb::AfbModImport!();

// include verb callbacks code & definitions
// -----------------------------------------
#[path = "./demo-verb-probe.rs"]
pub mod verb_probe;

#[path = "./demo-verb-basic.rs"]
pub mod verb_basic;

#[path = "./demo-verb-typed.rs"]
pub mod verb_typed;

#[path = "./demo-publish-subscribe.rs"]
pub mod pub_sub_group;

#[path = "./demo-group-event.rs"]
pub mod event_group;

#[path = "./demo-group-timer.rs"]
pub mod timer_group;

#[path = "./demo-group-subcall.rs"]
pub mod subcall_group;

#[path = "./demo-group-loa.rs"]
pub mod loa_group;

#[path = "./demo-group-hello.rs"]
pub mod helloworld_group;

// use API userdata to store event & timer static handle
pub struct ApiUserData {
    _any_data: &'static str,
}

// AfbApi userdata should implement AfbApiControls trait
// trait provides default callback for: config,ready,orphan,class,exit
impl AfbApiControls for ApiUserData {
    // api is loaded but not ready to be used, when defined binder send binding specific configuration
    fn config(&mut self, api: &AfbApi, config: AfbJsonObj) -> i32 {
        let _api_data = self; // self matches api_data
        afb_log_msg!(
            Notice,
            api,
            "--api-config api={} config={}",
            api.get_uid(),
            config
        );

        AFB_OK // returning -1 will abort binder process
    }

    // the API is created and ready. At this level user may subcall api(s) declare as dependencies
    fn start(&mut self, _api: &AfbApi) -> i32 {
        let _api_data = self; // self matches api_data
        AFB_OK
    }

    // mandatory for downcasting back to custom apidata object
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

// Binding init callback started at binding load time before any API exist
// -----------------------------------------
pub fn binding_init(rootv4: AfbApiV4, jconf: AfbJsonObj) -> i32 {
    afb_log_msg!(Notice, rootv4, "-- binding-init binding config={}", jconf);

    // create a new api
    let status= AfbApi::new("rust-api")
        .set_name("rust-api")
        .set_info("My first Rust API")
        .set_permission(AfbPermission::new("acl:rust"))
        .set_callback(Box::new(ApiUserData {_any_data: "skipail"}))
        .add_verb(verb_probe::register(rootv4))
        .add_verb(verb_basic::register(rootv4))
        .add_verb(verb_typed::register(rootv4))
        .add_group(event_group::register(rootv4))
        .add_group(timer_group::register(rootv4))
        .add_group(subcall_group::register(rootv4))
        .add_group(pub_sub_group::register(rootv4))
        .add_group(loa_group::register(rootv4))
        .add_group(helloworld_group::register(rootv4))
        .seal(false)
        .finalize();

    match status {
        Ok(api) => {
            afb_log_msg!(Notice, rootv4, "RUST api uid={} started", api.get_uid());
            AFB_OK
        }
        Err(error) => {
            afb_log_msg!(Critical, rootv4, "Fail to register api error={}", error);
            AFB_FATAL
        }
    }
}

// register binding within libafb
AfbBindingRegister!(binding_init);
