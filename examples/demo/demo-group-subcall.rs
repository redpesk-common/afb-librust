/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk samples code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 */

// import libafb dependencies
use libafb::prelude::*;

// subcall demo create a dummy "loop-test/ping" to enable loopback test
// both call ping verb and return result.

struct ASyncApiData {
    my_counter: u32,
}

// async response is s standard (AfbVerbRegister!) API/verb callback
AfbVerbRegister!(AsyncResponseCtrl, async_response_cb, ASyncApiData);
fn async_response_cb(request: &AfbRequest, params: &AfbData, userdata: &mut ASyncApiData) {
    userdata.my_counter += 1;

    // we expect 1st argument to be json compatible
    let jquery = match params.get::<AfbJsonObj>(0) {
        Ok(argument) => {
            afb_log_msg!(
                Info,
                request,
                "async_response count={} params={}",
                userdata.my_counter,
                argument
            );
            argument
        }
        Err(mut error) => {
            afb_log_msg!(Error, request, "async_response error={}", error);
            request.reply(afb_add_trace!(error), -1);
            return;
        }
    };

    // rebuilt a new json object with upcase value of initial one
    let query = jquery.to_string().to_uppercase();
    let jreply = AfbJsonObj::parse(query.as_str()).unwrap();

    request.reply(jreply, 0);
}

AfbVerbRegister!(AsyncCallCtrl, async_call_cb);
fn async_call_cb(request: &AfbRequest, _args: &AfbData) {
    match AfbSubCall::call_async(
        request,
        "loop-test",
        "ping",
        AFB_NO_DATA,
        Box::new(AsyncResponseCtrl { my_counter: 99 }),
    ) {
        Err(error) => {
            afb_log_msg!(Error, request, &error);
        }
        Ok(()) => {}
    };
}

AfbVerbRegister!(SyncCallCtrl, sync_call_cb);
fn sync_call_cb(request: &AfbRequest, _args: &AfbData) {
    match AfbSubCall::call_sync(request, "loop-test", "ping", AFB_NO_DATA) {
        Err(mut error) => {
            afb_log_msg!(Error, request, &error);
            request.reply(afb_add_trace!(error), -1)
        }
        Ok(response) => {
            let status = response.get_status();
            let _len = response.get_count();
            request.reply(response, status);
        }
    };
}

pub fn register(apiv4: AfbApiV4) -> &'static AfbGroup {
    // build verb name from Rust module name
    let mod_name = module_path!().split(':').last().unwrap();
    afb_log_msg!(Notice, apiv4, "Registering group={}", mod_name);

    match AfbApi::new("loop-test").finalize() {
        Ok(api_test) => {
            afb_log_msg!(Notice, apiv4, "Loopback api uid={} started", api_test.get_uid());
        }
        Err(error) => {
            afb_log_msg!(Critical, apiv4, "Fail to register api error={}", error);
            panic!("(hoops) fail to create loop-test")
        }
    };

    let job_post = AfbVerb::new("sync-call")
        .set_callback(Box::new(SyncCallCtrl {}))
        .set_info("synchronous call to internal loop-test/ping")
        .set_usage("no input")
        .finalize();

    let start_timer = AfbVerb::new("async-call")
        .set_callback(Box::new(AsyncCallCtrl {}))
        .set_info("asynchronous call to loop-test/ping")
        .set_usage("no input")
        .finalize();

    AfbGroup::new(mod_name)
        .set_info("timer demo api group")
        .set_prefix(mod_name)
        //.set_permission(AfbPermission::new("acl:evt"))
        .set_verbosity(3)
        .add_verb(job_post)
        .add_verb(start_timer)
        .finalize()
}
