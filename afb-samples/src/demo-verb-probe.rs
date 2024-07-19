/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk samples code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
*/

// import libafb dependencies
use afbv4::prelude::*;

// just return use to test afb-rust framework minimal cost
fn probe_callback(request: &AfbRequest, _args: &AfbRqtData, _ctx: &AfbCtxData) ->Result <(), AfbError> {
    request.reply(AFB_NO_DATA, 0);
    Ok(())
}

pub fn register(apiv4: AfbApiV4) ->  Result<&'static AfbVerb, AfbError> {
    // build verb name from Rust module name
    let mod_name= module_path!().split(':').last().unwrap();
    afb_log_msg!(Notice, apiv4, "Registering verb={}", mod_name);

    let group=AfbVerb::new(mod_name)
        .set_callback(probe_callback)
        .set_info("Probe no input/output data")
        .set_usage("no-data")
        .finalize()?;

    Ok(group)
}
