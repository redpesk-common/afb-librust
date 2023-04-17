/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk samples code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
*/

// import libafb dependencies
use libafb::prelude::*;

// just return use to test afb-rust framework minimal cost
AfbVerbRegister!(VerbCtrl, callback);
fn callback(request: &AfbRequest, _args: &AfbData) {
    request.reply(AFB_NO_DATA, 0);
}

pub fn register(apiv4: AfbApiV4) -> &'static AfbVerb {
    // build verb name from Rust module name
    let mod_name= module_path!().split(':').last().unwrap();
    afb_log_msg!(Notice, apiv4, "Registering verb={}", mod_name);

    AfbVerb::new(mod_name)
        .set_callback(Box::new(VerbCtrl {}))
        .set_info("Probe no input/output data")
        .set_usage("no-data")
        .finalize()
}
