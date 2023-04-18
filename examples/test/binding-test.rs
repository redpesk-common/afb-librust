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

// import demo SimpleData converter.
extern crate demo_converter;
use self::demo_converter::MySimpleData;

// import libafb dependencies
use libafb::prelude::*;

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

        // ------ Simple verb -----------
        let test1 = AfbTapTest::new("builtin-ping", "rust-api", "ping")
            .set_info("My simple ping test")
            .finalize()?;

        let test2 = AfbTapTest::new("jsonc-arg", "rust-api", "verb_basic")
            .set_info("Check json input param")
            .add_arg(&JsonStr(
                "{'skipail':'IoT.bzh','location':'Lorient','lander':'Brittany'}",
            ))?
            .finalize()?;

        let test3 = AfbTapTest::new("jsonc-reply", "rust-api", "verb_basic")
            .set_info("Check json response")
            .add_arg(&JsonStr(
                "{'skipail':'Follijen','location':'PortLouis','lander':'Brittany'}",
            ))
            .expect("valid argument")
            .add_expect(&JsonStr("{'LANDER':'BRITTANY'}"))
            .finalize()?;

        let test4 = AfbTapTest::new("jsonc-arg", "rust-api", "verb_typed")
            .set_info("Check invalid typed input")
            .add_arg(&JsonStr("{'x':1,'y':123,'name':'Skipail IoT.bzh'}"))?
            .finalize()?;

        let test5 = AfbTapTest::new("MySimpleData", "rust-api", "verb_typed")
            .set_info("Check invalid typed input")
            .add_arg(MySimpleData {
                x: 256,
                y: 1024,
                name: "Skipail IoT.bzh".to_owned(),
            })?
            .set_onsuccess("check-loa")
            .finalize()?;

        // ------ LOA Group -----------
        let loa1 = AfbTapTest::new("loa-check-x", "rust-api", "loa_group/check")
            .set_info("Check missing LOA fail with invalid scope")
            .set_status(-9)
            .finalize()?; // invalid scope

        let loa2 =
            AfbTapTest::new("loa-set-1", "rust-api", "loa_group/set").set_info("Set loa to 1").finalize()?;

        let loa3 = AfbTapTest::new("loa-check-1", "rust-api", "loa_group/check")
            .set_info("Check should work as session LOA now = 1")
            .set_onsuccess("check-timer")
            .finalize()?;

        let loa_group = AfbTapGroup::new("check-loa")
            .set_info("check session LOA")
            .add_test(loa1)?
            .add_test(loa2)?
            .add_test(loa3)?
            .finalize()?;

        // ------ Timer Group -----------
        let timer1 = AfbTapTest::new("break-timeout", "rust-api", "timer_group/job-post")
            .set_info("Check should fail in timeout")
            .set_timeout(1)
            .set_status(-62) // timeout
            .finalize()?;

        let timerx = AfbTapTest::new("builtin-ping", "rust-api", "ping")
            .set_info("My simple ping test")
            .finalize()?;

        let timer2 = AfbTapTest::new("response-3s", "rust-api", "timer_group/job-post")
            .set_info("Check should provide a response in 3s")
            .set_onsuccess("check-event")
            .finalize()?;

        let timer_group = AfbTapGroup::new("check-timer")
            .set_info("Check delay and timer")
            .add_test(timer1)?
            .add_test(timerx)?
            .add_test(timer2)?
            .finalize()?;

        // ------ Event Group -----------
        let event1 = AfbTapTest::new("event-subscribe", "rust-api", "event_group/subscribe")
            .set_info("subscribe to event").finalize()?;

        let event2 = AfbTapTest::new("event-push-one-listener", "rust-api", "event_group/push")
            .set_info("check event as 1 listener")
            .add_arg("{'info': 'some data event'}")?
            .finalize()?;

        let event3 = AfbTapTest::new("event-unsubscribe", "rust-api", "event_group/unsubscribe")
            .set_info("Unsubscribe event").finalize()?;

        let event4 = AfbTapTest::new("event-push-no-listener", "rust-api", "event_group/push")
            .set_info("push should not have any subscriber")
            .add_expect(0)
            .add_arg("{'info': 'some data event'}")?
            .finalize()?;

        let event_group = AfbTapGroup::new("check-event")
            .set_info("check session EVENT")
            .add_test(event1)?
            .add_test(event2)?
            .add_test(event3)?
            .add_test(event4)?
            .finalize()?;

        AfbTapSuite::new(api, "Tap Demo Test")
            .set_info("Check Example demo API works")
            .set_timeout(0)
            .add_test(test1)?
            .add_test(test2)?
            .add_test(test3)?
            .add_test(test4)?
            .add_test(test5)?
            .add_group(event_group)?
            .add_group(loa_group)?
            .add_group(timer_group)?
            .set_autorun(self.autostart)
            .set_autoexit(self.autoexit)
            .set_output(self.output)
            .finalize()?;

        Ok(())
    }

    fn config(&mut self, api: &AfbApi, jconf: AfbJsonObj) -> Result<(), AfbError> {
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
    demo_converter::register(rootv4)?;

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
