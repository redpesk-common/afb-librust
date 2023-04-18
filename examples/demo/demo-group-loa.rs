/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk samples code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 */

// import libafb dependencies
use libafb::prelude::*;

// loa test group expose 3 verbs
// -- loa/set: move session LOA to 1
// -- loa/reset: reset loa to zero
// --loa/check: request a loa>=1 to accept the request

AfbVerbRegister!(SetLoaCtrl, set_loa_cb);
fn set_loa_cb(request: &AfbRequest, _args: &AfbData) {
    match request.set_loa(1) {
        Err(mut error) => request.reply (afb_add_trace!(error), -1),
        Ok(loa) => request.reply(format!("LOA set to {}", loa), 0)
    }
}

AfbVerbRegister!(ResetLoaCtrl, reset_loa_cb);
fn reset_loa_cb(request: &AfbRequest, _args: &AfbData) {
    match request.set_loa(0) {
        Err(mut error) => request.reply (afb_add_trace!(error), -1),
        Ok(loa) => request.reply(format!("LOA reset to {}", loa), 0)
    }

    request.reply("LOA reset to 0", 0);
}

AfbVerbRegister!(CheckLoaCtrl, check_loa_cb);
fn check_loa_cb(request: &AfbRequest, _args: &AfbData) {
    request.reply("Protected API with LOA>=1 OK", 0);
}

// prefix group of event verbs and attach a default privilege
pub fn register(apiv4: AfbApiV4) -> Result<&'static AfbGroup, AfbError> {
    // build verb name from Rust module name
    let mod_name = module_path!().split(':').last().unwrap();
    afb_log_msg!(Notice, apiv4, "Registering group={}", mod_name);

    let reset = AfbVerb::new("reset")
        .set_callback(Box::new(ResetLoaCtrl {}))
        .set_info("Reset Loa to zero")
        .set_usage("no input")
        .finalize()?;

    let set = AfbVerb::new("set")
        .set_callback(Box::new(SetLoaCtrl {}))
        .set_info("Set Loa to 1")
        .set_permission(AfbPermission::new("acl:valeo"))
        .set_usage("no input")
        .finalize()?;

    let check = AfbVerb::new("check")
        .set_callback(Box::new(CheckLoaCtrl {}))
        .set_info("Request LOA>=1 to accept incoming request")
        .set_usage("no input")
        .set_permission(AfbPermission::new(1))
        .finalize()?;

    let group=AfbGroup::new(mod_name)
        .set_info("LOA demo group")
        .set_prefix(mod_name)
        .set_permission(AfbPermission::new("acl:loa"))
        .add_verb(set)?
        .add_verb(reset)?
        .add_verb(check)?
        .finalize()?;
    Ok(group)
}
