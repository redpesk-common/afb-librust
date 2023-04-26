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
extern crate libafb;
use std::time::Instant;

// import libafb dependencies
use libafb::prelude::*;

// import demo SimpleData converter.
extern crate demo_converter;
use self::demo_converter::MySimpleData;

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
        afb_log_msg!(Notice, api, "starting afb-rust type benchmark");

        let ping_test = AfbTapTest::new("no-data", "loop-bench", "nodata-convert")
            .set_info("Start 1000 loop without any data parameter")
            .add_arg(&JsonStr("{'loop':10000}"))?;

        let json_test = AfbTapTest::new("json-args", "loop-bench", "json-convert")
            .set_info("Start 1000 loop with binary parameter")
            .add_arg(&JsonStr("{'loop':10000}"))?;

        let lazy_test = AfbTapTest::new("binary-args", "loop-bench", "lazy-convert")
            .set_info("Start 1000 loop with binary parameter")
            .add_arg(&JsonStr("{'loop':10000}"))?;

        AfbTapSuite::new(api, "Tap Demo Test")
            .set_info("Benchmark afb-rust type converters")
            .set_timeout(0)
            .add_test(ping_test)
            .add_test(json_test)
            .add_test(lazy_test)
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

struct UserTimerData {
    count: u32,
    start: Instant,
    apiv4: AfbApiV4,
}

AfbTimerRegister!(TimerHandlerCtrl, timer_callback, UserTimerData);
fn timer_callback(timer: &AfbTimer, decount: u32, userdata: &mut UserTimerData) {
    match AfbSubCall::call_sync(userdata.apiv4, "rust-api", "verb_probe", AFB_NO_DATA) {
        Err(error) => {
            afb_log_msg!(Error, userdata.apiv4, &afb_add_trace!(error));
            timer.unref();
            return;
        }
        Ok(result) => {
            if result.get_status() != AFB_OK {
                afb_log_msg!(Error, userdata.apiv4, "status != AFB_OK");
                timer.unref();
                return;
            }
        }
    };

    if decount == 1 {
        // last run print result
        let msg = format!(
            "no-data loop:{} duration:{:?}",
            userdata.count,
            userdata.start.elapsed()
        );
        afb_log_msg!(Notice, userdata.apiv4, msg.as_str());
    }
}

// call API without any argument
AfbVerbRegister!(TimerVerbCtrl, timer_verb_cb);
fn timer_verb_cb(request: &AfbRequest, args: &AfbData) -> Result<(),AfbError> {

    let jargs= args.get::<JsoncObj>(0) ?;
    let count = jargs.get::<u32>("loop") ?;
    let tic = jargs.get::<u32>("tic") ?;

    let userdata = UserTimerData {
        start: Instant::now(),
        count: count,
        apiv4: request.get_api().get_apiv4(),
    };

    let timer = AfbTimer::new("demo_timer")
        .set_period(tic)
        .set_decount(count)
        .set_callback(Box::new(userdata))
        .start()
    {
        Err(error) => {
            afb_log_msg!(Critical, request, &error);
            return Err(error);
        }
        Ok(value) => value,
    };

    let msg = format!(
        "api:{} loop:{} timer:{}ms started",
        request.get_verb().get_uid(),
        count,
        tic
    );
    afb_log_msg!(Notice, request, msg.as_str());
    request.reply(msg.as_str(), 0);
    timer.addref(); // make sure timer remain acting after verb callback
    Ok(())
}

// call API without any argument
AfbVerbRegister!(PingCtrl, ping_subcall_cb);
fn ping_subcall_cb(request: &AfbRequest, args: &AfbData) -> Result<(), AfbError>{
    let jarg = args.get::<JsoncObj>(0)?;
    let count= jarg.get::<u32>("loop")?;

    let start = Instant::now();
    for _idx in 0..count {
        AfbSubCall::call_sync(request, "rust-api", "verb_probe", AFB_NO_DATA)?;
    }
    let duration = start.elapsed();
    let msg = format!("no-data loop:{} duration:{:?}", count, duration);
    afb_log_msg!(Notice, request, msg.as_str());
    request.reply(msg.as_str(), 0);
    Ok(())
}

// call API with passing a retrieve a jsonc object, this will force conversion
AfbVerbRegister!(JsonCtrl, json_subcall_cb);
fn json_subcall_cb(request: &AfbRequest, args: &AfbData) -> Result<(), AfbError> {
    let jargs = args.get::<JsoncObj>(0) ?;
    let count= jargs.get::<u32>("loop") ?;

    let start = Instant::now();
    for idx in 0..count {
        // just to prove we do not cache response
        let userdata = MySimpleData {
            x: idx as i32,
            y: idx as i32 * -1,
            name: "Skipail IoT.bzh".to_owned(),
        };

        // build a json object from binary object
        let json_data = JsoncObj::new();
        json_data.add("x", userdata.x).unwrap();
        json_data.add("y", userdata.y).unwrap();
        json_data.add("name", "Skipail IoT.bzh").unwrap();

        let param = AfbParams::from(json_data)?;

        let reply = AfbSubCall::call_sync(request, "rust-api", "verb_typed", param)?;

        match reply.get::<JsoncObj>(0) {
            Err(error) => {
                afb_log_msg!(Error, request, &error);
                return Err(error)
            }
            Ok(json_data) => {
                // use a data to assert data structure is valid
                let _x = json_data.get::<u32>("x");
                let _y = json_data.get::<u32>("y");
                let _name = json_data.get::<String>("name");
            }
        }
    }
    let duration = start.elapsed();
    let msg = format!("json converter loop:{} duration:{:?}", count, duration);
    afb_log_msg!(Notice, request, msg.as_str());
    request.reply(msg.as_str(), 0);
    Ok(())
}

// call API with passing a binary object using lazy converting mode
AfbVerbRegister!(LazyCtrl, lazy_subcall_cb);
fn lazy_subcall_cb(request: &AfbRequest, args: &AfbData) -> Result <(), AfbError> {
    let jargs = args.get::<JsoncObj>(0) ?;
    let count = jargs.get::<u32>("loop") ?;

    let start = Instant::now();
    for idx in 0..count {
        let userdata = MySimpleData {
            x: idx as i32,
            y: idx as i32 * -1,
            name: "Skipail IoT.bzh".to_owned(),
        };

        let param = AfbParams::from(userdata) ?;

        let reply= AfbSubCall::call_sync(request, "rust-api", "verb_typed", param) ?;
        match reply.get::<&MySimpleData>(0) {
            Err(error) => {
                afb_log_msg!(Error, request, &error);
                return Err(error);
            }
            Ok(simple_data) => {
                // use a data to assert data structure is valid
                let _x = simple_data.x;
                let _y = simple_data.y;
                let _name = simple_data.name.as_str();
            }
        }
    }
    let duration = start.elapsed();
    let msg = format!("direct converter loop:{} duration:{:?}", count, duration);
    afb_log_msg!(Notice, request, msg.as_str());
    request.reply(msg.as_str(), 0);
    Ok(())
}

// rootv4 init callback started at rootv4 load time before any API exist
// -----------------------------------------
pub fn binding_test_init(rootv4: AfbApiV4, jconf: JsoncObj) -> Result<&'static AfbApi, AfbError> {
    let uid = match jconf.get::<String>("uid") {
        Ok(value) => value,
        Err(_error) => "afbrust-bench-rootv4".to_owned(),
    };

    let tap_config = TapUserData {
        autostart: true,
        autoexit: true,
        output: AfbTapOutput::TAP,
    };

    // custom type should register once per binder
    demo_converter::register(rootv4).expect("must register custom type");
    let timer_nodata = AfbVerb::new("timer-no-data")
        .set_callback(Box::new(TimerVerbCtrl {}))
        .set_info("No argument")
        .set_usage("{'loop': ?,'tic':?ms}")
        .set_sample("{'loop':1000,'tic':10}")
        .expect("invalid json")
        .finalize()?;

    let ping_verb = AfbVerb::new("nodata-convert")
        .set_callback(Box::new(PingCtrl {}))
        .set_info("No argument")
        .set_usage("{'loop': xxx}")
        .set_sample("{'loop': 100}")
        .expect("invalid json")
        .set_sample("{'loop': 1000}")
        .expect("invalid json")
        .finalize()?;

    let lazy_verb = AfbVerb::new("lazy-convert")
        .set_callback(Box::new(LazyCtrl {}))
        .set_info("Direct binary argument")
        .set_usage("{'loop': xxx}")
        .set_sample("{'loop': 100}")
        .expect("invalid json")
        .set_sample("{'loop': 1000}")
        .expect("invalid json")
        .finalize()?;

    let json_verb = AfbVerb::new("json-convert")
        .set_callback(Box::new(JsonCtrl {}))
        .set_info("Json argument conversion ")
        .set_usage("{'loop': xxx}")
        .set_sample("{'loop': 100}")
        .expect("invalid json")
        .set_sample("{'loop': 1000}")
        .expect("invalid json")
        .finalize()?;

    // create a loop api to initiate subcalls
    AfbApi::new("loop-bench")
        .set_info("Loopback api used to initiate subcalls")
        .add_verb(ping_verb)
        .add_verb(json_verb)
        .add_verb(lazy_verb)
        .add_verb(timer_nodata)
        .finalize()?;

    // create test api for automatic benchmarking
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
