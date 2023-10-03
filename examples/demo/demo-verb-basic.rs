/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk samples code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
*/

// import libafb dependencies
use afb_rust::prelude::*;

// AfbApi AfbVerb without vcbdata
AfbVerbRegister!(VerbCtrl, callback);
fn callback(request: &AfbRequest, args: &AfbData) -> Result<(), AfbError> {
    let jquery = args.get::<JsoncObj>(0)?;

    // rebuilt a new json object with upcase value of initial one
    let data = jquery.to_string().to_uppercase();
    let jreply = JsoncObj::parse(data.as_str()).unwrap();

    let reply = || -> Result<(), AfbError> {
        let mut response = AfbParams::new();
        response.push(jreply)?;
        request.reply(response, 0);
        Ok(())
    };

    // if data export fail send an error report
    if let Err(error) = reply() {
        return Err(error);
    }
    Ok(())
}

pub fn register(apiv4: AfbApiV4) -> Result<&'static AfbVerb, AfbError> {
    // build verb name from Rust module name
    let mod_name = module_path!().split(':').last().unwrap();
    afb_log_msg!(Notice, apiv4, "Registering verb={}", mod_name);

    let verb = AfbVerb::new(mod_name)
        .set_callback(Box::new(VerbCtrl {}))
        .set_info("My 1st demo verb")
        .set_usage("any json string")
        .set_sample("{'skipail': 'IoT.bzh', 'location':'Lorient'}")?
        .finalize()?;
    Ok(verb)
}
