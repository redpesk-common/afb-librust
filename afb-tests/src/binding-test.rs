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

// import libafb dependencies
use afbv4::prelude::*;
use afb_converter::MySimpleData;

struct ASyncCallData {
    my_counter: u32,
}

// async response is s standard (AfbVerbRegister!) API/verb callback
fn async_response_cb(api: &AfbApi, params: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let ctx = ctx.get_mut::<ASyncCallData>()?;
    ctx.my_counter += 1;

    // we expect 1st argument to be json compatible
    match params.get::<JsoncObj>(0) {
        Ok(argument) => {
            afb_log_msg!(
                Notice,
                api,
                "async_response count={} params={}",
                ctx.my_counter,
                argument
            );
        }
        Err(error) => {
            afb_log_msg!(Error, api, "async_response error={}", error);
            return Err(error);
        }
    };
    Ok(())
}

// This rootv4 demonstrate how to test an external rootv4 that you load within the same afb-binder process and security context
// It leverages test (Test Anything Protocol) that is compatible with redpesk testing report.
struct TapUserData {
    autostart: bool,
    autoexit: bool,
    output: AfbTapOutput,
}

// AfbApi userdata implements AfbApiControls trait
impl AfbApiControls for TapUserData {
    fn start(&mut self, api: &AfbApi) -> Result<(), AfbError> {
        afb_log_msg!(Notice, api, "starting TAP testing");

        // testing subcall async
        AfbSubCall::call_async(
            api,
            "rust-api",
            "ping",
            AFB_NO_DATA,
            async_response_cb,
            ASyncCallData { my_counter: 0 },
        )?;

        // ------ Simple verb -----------
        let test0 = AfbTapTest::new("builtin-info", "rust-api", "info")
            .set_info("My simple info test")
            .finalize()?;

        let test1 = AfbTapTest::new("builtin-ping", "rust-api", "ping")
            .set_info("My simple ping test")
            .finalize()?;

        let test2 = AfbTapTest::new("jsonc-basic", "rust-api", "verb_basic")
            .set_info("Check json input param")
            .add_arg(
                "{'skipail':'IoT.bzh','location':'Lorient','lander':'Brittany'}",
            )?
            .finalize()?;

        let test3 = AfbTapTest::new("jsonc-reply", "rust-api", "verb_basic")
            .set_info("Check json response")
            .add_arg("{'skipail':'Follijen','location':'PortLouis','lander':'Brittany'}")?
            .add_expect("{'LANDER':'BRITTANY'}")?
            .finalize()?;

        let test4 = AfbTapTest::new("jsonc-typed", "rust-api", "verb_typed")
            .set_info("Check invalid typed input")
            .add_arg("{'x':1,'y':123,'name':'Skipail IoT.bzh'}")?
            .finalize()?;

        let test5 = AfbTapTest::new("MySimpleData", "rust-api", "verb_typed")
            .set_info("Check custom typed input")
            .add_arg(MySimpleData {
                x: 256,
                y: 1024,
                name: "Skipail IoT.bzh".to_owned(),
            })?
            .set_onsuccess("check-session")
            .finalize()?;

        // ------ SESSION Group -----------
        let rqt1 = AfbTapTest::new("session-check1", "rust-api", "session_group/reset")
            .set_info("Create a new session")
            .add_expect(0)?
            .finalize()?;

        let rqt2 = AfbTapTest::new("session-check2", "rust-api", "session_group/read")
            .set_info("Read session")
            .add_expect(1)?
            .finalize()?;

        let rqt3 = AfbTapTest::new("session-check3", "rust-api", "session_group/read")
            .set_info("Read session")
            .add_expect(2)?
            .finalize()?;

        let rqt4 = AfbTapTest::new("session-check4", "rust-api", "session_group/reset")
            .set_info("Reset session")
            .add_expect(0)?
            .finalize()?;

        let rqt5 = AfbTapTest::new("session-check5", "rust-api", "session_group/read")
            .set_info("Read new session")
            .add_expect(1)?
            .finalize()?;

        let rqt6 = AfbTapTest::new("session-check6", "rust-api", "session_group/drop")
            .set_info("Drop current session")
            .set_onsuccess("check-loa")
            .finalize()?;

        let session_group = AfbTapGroup::new("check-session")
            .set_info("check session RQT")
            .add_test(rqt1)
            .add_test(rqt2)
            .add_test(rqt3)
            .add_test(rqt4)
            .add_test(rqt5)
            .add_test(rqt6)
            .finalize()?;

        // ------ LOA Group -----------
        let loa1 = AfbTapTest::new("loa-check-x", "rust-api", "loa_group/check")
            .set_info("Check missing LOA fail with invalid scope")
            .set_status(-9)
            .finalize()?; // invalid scope

        let loa2 = AfbTapTest::new("loa-set-1", "rust-api", "loa_group/set")
            .set_info("Set loa to 1")
            .finalize()?;

        let loa3 = AfbTapTest::new("loa-check-1", "rust-api", "loa_group/check")
            .set_info("Check should work as session LOA now = 1")
            .set_onsuccess("check-timer")
            .finalize()?;

        let loa_group = AfbTapGroup::new("check-loa")
            .set_info("check session LOA")
            .add_test(loa1)
            .add_test(loa2)
            .add_test(loa3)
            .finalize()?;

        // ------ Timer Group -----------
        let timer1 = AfbTapTest::new("break-timeout", "rust-api", "timer_group/job-post")
            .set_info("Check should fail in timeout")
            .set_timeout(1)
            .set_status(-62) // timeout
            .finalize()?;

        let timer2 = AfbTapTest::new("response-3s", "rust-api", "timer_group/job-post")
            .set_info("Check should provide a response in 3s")
            .set_onsuccess("check-event")
            .finalize()?;

        let timer_group = AfbTapGroup::new("check-timer")
            .set_info("Check delay and timer")
            .add_test(timer1)
            .add_test(timer2)
            .finalize()?;

        // ------ Event Group -----------
        let event1 = AfbTapTest::new("event-subscribe", "rust-api", "event_group/subscribe")
            .set_info("subscribe to event")
            .finalize()?;

        let event2 = AfbTapTest::new("event-push-one-listener", "rust-api", "event_group/push")
            .set_info("check event as 1 listener")
            .add_arg("{'info': 'some data event'}")?
            .add_expect(1)? // on listener
            .finalize()?;

        let event3 = AfbTapTest::new("event-unsubscribe", "rust-api", "event_group/unsubscribe")
            .set_info("Unsubscribe event")
            .finalize()?;

        let event4 = AfbTapTest::new("event-push-no-listener", "rust-api", "event_group/push")
            .set_info("push should not have any subscriber/session")
            .add_arg("{'info': 'some data event'}")?
            .set_status(-100) // no more session
            .finalize()?;

        let event_group = AfbTapGroup::new("check-event")
            .set_info("check session EVENT")
            .add_test(event1)
            .add_test(event2)
            .add_test(event3)
            .add_test(event4)
            .finalize()?;

        AfbTapSuite::new(api, "Tap Demo Test")
            .set_info("Check Example demo API works")
            .set_timeout(0)
            .add_test(test0)
            .add_test(test1)
            .add_test(test2)
            .add_test(test3)
            .add_test(test4)
            .add_test(test5)
            .add_group(session_group)
            .add_group(event_group)
            .add_group(loa_group)
            .add_group(timer_group)
            .set_autorun(self.autostart)
            .set_autoexit(self.autoexit)
            .set_output(self.output)
            .finalize()?;

        Ok(())
    }

    fn config(&mut self, api: &AfbApi, jconf: JsoncObj) -> Result<(), AfbError> {
        afb_log_msg!(Debug, api, "api={} config={}", api.get_uid(), jconf);
        match jconf.get::<bool>("autostart") {
            Ok(value) => self.autostart = value,
            Err(_error) => {}
        };

        match jconf.get::<bool>("autoexit") {
            Ok(value) => self.autoexit = value,
            Err(_error) => {}
        };

        match jconf.get::<String>("output") {
            Err(_error) => {}
            Ok(value) => match value.to_uppercase().as_str() {
                "JSON" => self.output = AfbTapOutput::JSON,
                "TAP" => self.output = AfbTapOutput::TAP,
                "NONE" => self.output = AfbTapOutput::NONE,
                _ => {
                    afb_log_msg!(
                        Error,
                        api,
                        "Invalid output should be json|tap (default used)"
                    );
                }
            },
        };

        Ok(())
    }

    // mandatory for downcasting back to custom apidata object
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

// rootv4 init callback started at rootv4 load time before any API exist
// -----------------------------------------
pub fn binding_test_init(rootv4: AfbApiV4, jconf: JsoncObj) -> Result<&'static AfbApi, AfbError> {
    let uid = match jconf.get::<String>("uid") {
        Ok(value) => value,
        Err(_error) => "Tap-test-rootv4".to_owned(),
    };

    let tap_config = TapUserData {
        autostart: true,
        autoexit: true,
        output: AfbTapOutput::TAP,
    };

    // custom type should register once per binder
    afb_converter::register(rootv4)?;

    afb_log_msg!(Notice, rootv4, "-- rootv4 {} loaded", uid);
    let api = AfbApi::new("tap-test")
        .set_info("Testing Tap reporting")
        .require_api("rust-api")
        .set_callback(Box::new(tap_config))
        .seal(false)
        .finalize()?;
    Ok(api)
}

// register rootv4 within libafb
AfbBindingRegister!(binding_test_init);
