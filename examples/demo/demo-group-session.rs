/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk samples code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 */

// import libafb dependencies
use libafb::prelude::*;

// session user data (one private instance per client)
AfbSessionRegister!(SessionUserData, session_drop_cb);
struct SessionUserData {
    count: u32,
}

fn session_drop_cb(session: &mut SessionUserData) {
    println!("*** session closing count={} ***", session.count);
}

AfbVerbRegister!(CreateCtrl, create_callback);
fn create_callback(request: &AfbRequest, _args: &AfbData)  -> Result <(), AfbError>  {
    let session= SessionUserData::set(request, SessionUserData{count:0})?;
    request.reply(session.count, 0);
    Ok(())
}

AfbVerbRegister!(DropCtrl, drop_callback);
fn drop_callback(request: &AfbRequest, _args: &AfbData)  -> Result <(), AfbError> {
    SessionUserData::unref(request)?;
    request.reply(AFB_NO_DATA, 0);
    Ok(())
}

AfbVerbRegister!(GetCtrl, get_callback);
fn get_callback(request: &AfbRequest, _args: &AfbData)  -> Result <(), AfbError> {
    let session = SessionUserData::get(request)?;
    session.count += 1;
    request.reply(session.count, 0);
    Ok(())
}

// prefix group of event verbs and attach a default privilege
pub fn register(apiv4: AfbApiV4) -> Result<&'static AfbGroup, AfbError> {
    // build verb name from Rust module name
    let mod_name = module_path!().split(':').last().unwrap();
    afb_log_msg!(Notice, apiv4, "Registering group={}", mod_name);

    let drop = AfbVerb::new("drop")
        .set_info("drop session context")
        .set_callback(Box::new(DropCtrl{}))
        .set_usage("no input")
        .finalize()?;

    let create = AfbVerb::new("set/reset")
        .set_name("reset")
        .set_info("create a new session context")
        .set_usage("no input")
        .set_callback(Box::new(CreateCtrl))
        .finalize()?;

    let read = AfbVerb::new("read")
        .set_info("read session context")
        .set_usage("no input")
        .set_callback(Box::new(GetCtrl))
        .finalize()?;

    let group = AfbGroup::new(mod_name)
        .set_info("session demo group")
        .set_prefix(mod_name)
        .set_permission(AfbPermission::new("acl:evt"))
        .set_verbosity(3)
        .add_verb(create)
        .add_verb(drop)
        .add_verb(read)
        .finalize()?;
    Ok(group)
}
