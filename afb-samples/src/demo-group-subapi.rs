/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk samples code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 */

// import libafb dependencies
use afbv4::prelude::*;

// note: in production a unique API/verb should do both timer creation and event subscription
fn hello_stop_cb(
    request: &AfbRequest,
    _args: &AfbRqtData,
    _ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    match AfbSubCall::call_sync(request, "helloworld-event", "unsubscribe", AFB_NO_DATA) {
        Err(error) => {
            afb_log_msg!(Error, request, &error);
            request.reply(afb_add_trace!(error), -1);
        }
        Ok(_response) => {}
    };
    Ok(())
}

// async subcall response behaves as any other API/verb callback
fn hello_response_cb(
    request: &AfbRequest,
    _params: &AfbRqtData,
    _ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    request.reply(
        "subscribe helloworld done (check log in afb-binder console)",
        0,
    );
    Ok(())
}

// Start helloworld timer in synchronous mode and for the fun subscribe to event in asynchronous mode
// note: in production a unique API/verb should do both timer creation and event subscription
fn hello_start_cb(
    request: &AfbRequest,
    _args: &AfbRqtData,
    _ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    AfbSubCall::call_sync(request, "helloworld-event", "startTimer", AFB_NO_DATA)?;

    AfbSubCall::call_async(
        request,
        "helloworld-event",
        "subscribe",
        AFB_NO_DATA,
        hello_response_cb,
        AFB_NO_DATA,
    )?;
    Ok(())
}

// prefix group of event verbs and attach a default privilege
pub fn register(apiv4: AfbApiV4) -> Result<&'static AfbGroup, AfbError> {
    // build verb name from Rust module name
    let mod_name = module_path!().split(':').next_back().unwrap();
    afb_log_msg!(Notice, apiv4, "Registering group={}", mod_name);

    let start_hello = AfbVerb::new("hello-start")
        .set_callback(hello_start_cb)
        .set_info("connect to helloworld &api start-timer and subscribe to event")
        .set_usage("no input")
        .finalize()?;

    let stop_hello = AfbVerb::new("hello-stop")
        .set_callback(hello_stop_cb)
        .set_info("asynchronous call to api-test/ping")
        .set_usage("no input")
        .finalize()?;

    let group = AfbGroup::new(mod_name)
        .set_info("timer demo api group")
        .set_prefix(mod_name)
        .set_permission(AfbPermission::new("acl:evt"))
        .set_verbosity(3)?
        .add_verb(start_hello)
        .add_verb(stop_hello)
        .finalize()?;

    Ok(group)
}
