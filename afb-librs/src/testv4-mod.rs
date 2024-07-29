/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * $RP_BEGIN_LICENSE$
 * Commercial License Usage
 *  Licensees holding valid commercial IoT.bzh licenses may use this file in
 *  accordance with the commercial license agreement provided with the
 *  Software or, alternatively, in accordance with the terms contained in
 *  a written agreement between you and The IoT.bzh Company. For licensing terms
 *  and conditions see https://www.iot.bzh/terms-conditions. For further
 *  information use the contact form at https://www.iot.bzh/contact.
 *
 * GNU General Public License Usage
 *  Alternatively, this file may be used under the terms of the GNU General
 *  Public license version 3. This license is as published by the Free Software
 *  Foundation and appearing in the file LICENSE.GPLv3 included in the packaging
 *  of this file. Please review the following information to ensure the GNU
 *  General Public License requirements will be met
 *  https://www.gnu.org/licenses/gpl-3.0.html.
 * $RP_END_LICENSE$
 */
use crate::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

const AUTOSTART: &str = "autostart";
const TIMEOUT: u32 = 5;

pub struct AfbTapResponse {
    pub status: i32,
    pub diagnostic: String,
}

pub struct AfbTapTest {
    pub uid: &'static str,
    pub info: &'static str,
    pub api: &'static str,
    pub verb: &'static str,
    pub status: i32,
    pub params: AfbParams,
    pub expect: Vec<JsoncObj>,
    pub onerror: Option<&'static str>,
    pub onsucess: Option<&'static str>,
    pub response: Option<AfbTapResponse>,
    pub timeout: u32,
    pub delay: u32,
    pub index: usize,
    semaphore: Arc<(Mutex<u32>, Condvar)>,
    group: *const AfbTapGroup,
}

struct TestAsyncCallCtx {
    // Returned arguments of the verb
    args: Option<AfbRqtData>,
}

// Function called at the end of a verb execution, in async
fn test_async_call_cb(_api: &AfbApi, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let shared_ctx = ctx.get_ref::<Arc<(Mutex<TestAsyncCallCtx>, Condvar)>>()?;
    let data = &shared_ctx.0;
    let cvar = &shared_ctx.1;

    let mut ret = data.lock().unwrap();

    ret.args = Some(args.clone());

    cvar.notify_one();

    ctx.free::<Arc<(Mutex<TestAsyncCallCtx>, Condvar)>>();

    Ok(())
}

impl AfbTapTest {
    pub fn new(
        uid: &'static str,
        api: &'static str,
        verb: &'static str,
    ) -> &'static mut AfbTapTest {
        let boxe = Box::new(AfbTapTest {
            uid: uid,
            info: "",
            api: api,
            verb: verb,
            status: 0,
            params: AfbParams::new(),
            expect: Vec::new(),
            onerror: None,
            onsucess: None,
            response: None,
            index: 0,
            timeout: 0,
            delay: 0,
            semaphore: Arc::new((Mutex::new(0), Condvar::new())),
            group: 0 as *mut AfbTapGroup,
        });
        Box::leak(boxe)
    }

    pub fn set_info(&mut self, value: &'static str) -> &mut Self {
        self.info = value;
        self
    }

    #[track_caller]
    pub fn finalize(&mut self) -> Result<&mut Self, AfbError> {
        Ok(self)
    }

    pub fn set_response(&mut self, status: i32, diagnostic: &str) -> &mut Self {
        self.response = Some(AfbTapResponse {
            status: status,
            diagnostic: diagnostic.to_owned(),
        });
        self
    }

    pub fn set_timeout(&mut self, value: u32) -> &mut Self {
        self.timeout = value;
        self
    }

    pub fn set_delay(&mut self, value: u32) -> &mut Self {
        self.delay = value;
        self
    }

    pub fn set_onsuccess(&mut self, group: &'static str) -> &mut Self {
        self.onsucess = Some(group);
        self
    }

    pub fn set_onerror(&mut self, group: &'static str) -> &mut Self {
        self.onerror = Some(group);
        self
    }

    pub fn set_status(&mut self, value: i32) -> &mut Self {
        self.status = value;
        self
    }

    #[track_caller]
    pub fn add_arg<T>(&mut self, param: T) -> Result<&mut Self, AfbError>
    where
        AfbParams: ConvertResponse<T>,
    {
        match self.params.push(param) {
            Err(error) => Err(error),
            Ok(_data) => Ok(self),
        }
    }

    pub fn add_expect<T>(&mut self, data: T) -> Result<&mut Self, AfbError>
    where
        JsoncObj: JsoncImport<T>,
    {
        let jvalue = JsoncObj::import(data)?;
        self.expect.push(jvalue);
        Ok(self)
    }

    pub fn get_group(&self) -> &AfbTapGroup {
        unsafe { &*(self.group as *mut AfbTapGroup) }
    }

    pub fn get_suite(&self) -> &AfbTapSuite {
        let group = unsafe { &*(self.group as *mut AfbTapGroup) };
        group.get_suite()
    }

    fn check_response(&self, reply: &AfbRqtData) -> AfbTapResponse {
        let api = self.get_suite().get_api();

        if reply.get_status() != self.status {
            let msg = format!(
                "status={} info={}",
                reply.get_status(),
                afb_error_info(reply.get_status())
            );
            return AfbTapResponse {
                status: AFB_FAIL,
                diagnostic: msg,
            };
        }

        for idx in 0..self.expect.len() {
            let jexpect = self.expect[idx].clone();
            match reply.get::<JsoncObj>(idx) {
                // expect argument as no jsonc representation.
                Err(error) => {
                    let msg = error.to_jsonc().unwrap().to_string();
                    return AfbTapResponse {
                        status: AFB_FAIL,
                        diagnostic: msg,
                    };
                }
                Ok(mut jvalue) => {
                    let jtest = if jexpect.is_type(Jtype::Object) {
                        jvalue.contains(jexpect.clone())
                    } else {
                        jvalue.equal(jexpect.clone())
                    };

                    match jtest {
                        Err(error) => {
                            afb_log_msg!(Warning, api, "{} -> {} NotIn {}", error, jexpect, jvalue);
                            return AfbTapResponse {
                                status: AFB_FAIL,
                                diagnostic: error.to_string(),
                            };
                        }
                        Ok(()) => {}
                    }
                }
            }
        }
        AfbTapResponse {
            status: AFB_OK,
            diagnostic: "".to_owned(),
        }
    }

    /// Call the configured verb with an optional timeout
    fn call_with_timeout(&mut self, timeout_s: u32) {
        let api = self.get_suite().get_api();
        afb_log_msg!(
            Info,
            api,
            "call_with_timeout idx:{} tap->uid:{} afb-api->'/{}/{}' timeout_s: {}",
            self.index,
            self.uid,
            self.api,
            self.verb,
            timeout_s
        );

        let ctx = TestAsyncCallCtx { args: None };
        let shared_ctx = Arc::new((Mutex::new(ctx), Condvar::new()));

        let response = AfbSubCall::call_async(
            api,
            self.api,
            self.verb,
            self.params.clone(),
            test_async_call_cb,
            shared_ctx.clone(),
        );
        if let Err(error) = response {
            let response = self.check_response(&AfbRqtData::without_data(error.get_status()));
            self.done(response);
            return;
        }

        let response = {
            let (lock, cvar) = &*shared_ctx;
            let mut ctx = lock.lock().unwrap();

            if timeout_s > 0 {
                // if a timeout is defined, we wait with timeout on the condition variable
                let result = cvar
                    .wait_timeout(ctx, Duration::from_millis((timeout_s * 1000) as u64))
                    .unwrap();
                if result.1.timed_out() {
                    self.check_response(&AfbRqtData::without_data(-62))
                } else {
                    let ctx = result.0;
                    let reply = ctx.args.as_ref().unwrap();
                    self.check_response(reply)
                }
            } else {
                while ctx.args.is_none() {
                    ctx = cvar.wait(ctx).unwrap();
                }
                let reply = ctx.args.as_ref().unwrap();
                self.check_response(reply)
            }
        };
        self.done(response);
    }

    // next_test should be executed within group thread context in order not to pile test execution
    fn get_next(&self) -> Option<&mut AfbTapTest> {
        let group = self.get_group();
        let suite = self.get_suite();

        // wait for job to be finish
        let (lock, cvar) = &*self.semaphore;
        let mut done = match lock.lock() {
            Err(_error) => return None,
            Ok(mutex) => mutex,
        };
        while *done != 0 {
            done = cvar.wait(done).unwrap();
        }

        let response = match &self.response {
            None => panic!("next-test require some response"),
            Some(value) => value,
        };

        if response.status == AFB_OK {
            match self.onsucess {
                None => group.get_test(self.index),
                Some(label) => match suite.get_group(label) {
                    None => None,
                    Some(next_group) => {
                        let next_test = next_group.get_test(0);
                        next_test
                    }
                },
            }
        } else {
            match self.onerror {
                None => group.get_test(self.index),
                Some(label) => match suite.get_group(label) {
                    None => None,
                    Some(group) => group.get_test(0),
                },
            }
        }
    }
    #[track_caller]
    pub fn jobpost(&mut self) -> Result<(), AfbError> {
        {
            let semaphore = Arc::clone(&self.semaphore);
            let (lock, cvar) = &*semaphore;
            let mut done = match lock.lock() {
                Err(_error) => {
                    return afb_error!("fail-group-wait", "fail waiting on tap group={}", self.uid)
                }
                Ok(mutex) => mutex,
            };
            *done = 1;
            cvar.notify_one();
        }

        // use group timeout as default
        let timeout = if self.timeout == 0 {
            self.get_group().timeout
        } else {
            self.timeout
        };

        self.call_with_timeout(timeout);

        Ok(())
    }

    pub fn done(&mut self, response: AfbTapResponse) {
        self.response = Some(response);
        let semaphore = Arc::clone(&self.semaphore);
        let (lock, cvar) = &*semaphore;
        let mut done = lock.lock().unwrap();
        *done = 0;
        cvar.notify_one();
    }

    pub fn get_report(&self) -> Result<JsoncObj, AfbError> {
        let msg = match &self.response {
            None => {
                format!("ok {} - {} # SKIP", self.index, self.uid)
            }
            Some(response) => {
                if response.status == AFB_OK {
                    format!("ok {} - {}", self.index, self.uid)
                } else {
                    format!(
                        "not ok {} - {} # {}",
                        self.index, self.uid, response.diagnostic
                    )
                }
            }
        };
        JsoncObj::import(msg.as_str())
    }
}

pub struct AfbTapGroup {
    pub uid: &'static str,
    pub info: &'static str,
    pub tests: Vec<*mut AfbTapTest>,
    index: usize,
    pub timeout: u32,
    suite: *mut AfbTapSuite,
    api_group: *mut AfbGroup,
}

impl AfbTapGroup {
    pub fn new(uid: &'static str) -> &'static mut AfbTapGroup {
        let boxe = Box::new(AfbTapGroup {
            uid: uid,
            info: "",
            timeout: 0,
            tests: Vec::new(),
            index: 0,
            suite: 0 as *mut AfbTapSuite,
            api_group: AfbGroup::new(uid),
        });
        Box::leak(boxe)
    }
    #[track_caller]
    pub fn finalize(&mut self) -> Result<&mut Self, AfbError> {
        Ok(self)
    }

    pub fn set_info(&'static mut self, value: &'static str) -> &'static mut Self {
        self.info = value;
        self
    }

    pub fn set_timeout(&'static mut self, value: u32) -> &'static mut Self {
        self.timeout = value;
        self
    }

    pub fn add_test(&'static mut self, test: &'static mut AfbTapTest) -> &'static mut Self {
        self.index += 1;
        test.index = self.index;
        test.group = self;
        self.tests.push(test);

        let verb = AfbVerb::new(test.uid)
            .set_info(test.info)
            .set_callback(tap_test_callback)
            .set_context(TapTestData { test: test })
            .set_usage("no input")
            .finalize()
            .unwrap();

        let api_group = unsafe { &mut *(self.api_group as *mut AfbGroup) };
        api_group.add_verb(verb);
        self
    }

    // return suite test until group end
    pub fn get_test(&self, index: usize) -> Option<&mut AfbTapTest> {
        if self.tests.len() <= index {
            None
        } else {
            let test = unsafe { &mut *(self.tests[index] as *mut AfbTapTest) };
            Some(test)
        }
    }

    pub fn get_suite(&self) -> &AfbTapSuite {
        unsafe { &*(self.suite as *const _ as *mut AfbTapSuite) }
    }
    #[track_caller]
    pub fn launch(&self) -> Result<(), AfbError> {
        // get group 1st test
        let test = match self.get_test(0) {
            None => return afb_error!(self.uid, "no-test-found"),
            Some(value) => value,
        };

        // launch test and wait for completion
        test.jobpost()?;

        // wait for jobpost completion before moving to next one
        let mut next = test.get_next();

        // callback return normaly or timeout
        while let Some(test) = next {
            test.jobpost()?;
            next = test.get_next();
        }
        Ok(())
    }

    pub fn get_report(&self) -> Result<JsoncObj, AfbError> {
        let jsonc = JsoncObj::array();
        let count = self.tests.len();
        let msg = format!("1..{} # {}", count, self.uid);
        jsonc.append(msg.as_str()).unwrap();
        for idx in 0..count {
            let test_ref = self.get_test(idx).unwrap();
            let test = &mut *(test_ref);
            jsonc.append(test.get_report()?)?;
        }
        Ok(jsonc)
    }
}

#[derive(Copy, Clone)]
pub enum AfbTapOutput {
    JSON,
    TAP,
    NONE,
}

pub struct AfbTapSuite {
    pub uid: &'static str,
    pub info: &'static str,
    pub autostart: *mut AfbTapGroup,
    pub autorun: bool,
    pub autoexit: bool,
    pub timeout: u32,
    pub output: AfbTapOutput,
    hashmap: HashMap<&'static str, *mut AfbTapGroup>,
    tap_api: *const AfbApi,
    event: *mut AfbEvent,
}

impl AfbTapSuite {
    pub fn new(api: &AfbApi, uid: &'static str) -> &'static mut AfbTapSuite {
        let mut hashmap: HashMap<&'static str, *mut AfbTapGroup> = HashMap::new();
        let autostart = AfbTapGroup::new(AUTOSTART);
        hashmap.insert(AUTOSTART, autostart);

        // register an event to notify test progression in api mode
        let event = AfbEvent::new(uid);
        event.register(api.get_apiv4());

        let boxe = Box::new(AfbTapSuite {
            uid: uid,
            info: "",
            autostart: autostart,
            autorun: true,
            autoexit: true,
            output: AfbTapOutput::TAP,
            timeout: TIMEOUT,
            hashmap: hashmap,
            tap_api: api,
            event: event,
        });

        // link autostart is default group to suite
        let suite = Box::leak(boxe);
        autostart.suite = suite;
        suite
    }

    pub fn set_timeout(&'static mut self, value: u32) -> &'static mut Self {
        self.timeout = value;
        self
    }

    pub fn add_test(&'static mut self, test: &'static mut AfbTapTest) -> &'static mut Self {
        let autostart = unsafe { &mut *(self.autostart) };
        if test.timeout == 0 {
            test.timeout = self.timeout;
        }

        autostart.add_test(test);
        self
    }

    pub fn add_group(&'static mut self, group: &mut AfbTapGroup) -> &'static mut Self {
        if group.timeout == 0 {
            group.timeout = self.timeout;
        }
        group.suite = self;
        self.hashmap.insert(group.uid, group);

        // create group test verb
        let vcbdata = TapGroupData { group: group };
        let api = unsafe { &mut *(self.tap_api as *mut AfbApi) };
        let verb = AfbVerb::new(group.uid);
        verb.set_callback(tap_group_callback)
            .set_context(vcbdata)
            .set_info(group.info)
            .set_usage("no_input")
            .register(api.get_apiv4(), AFB_NO_AUTH);
        api.add_verb(verb.finalize().unwrap());

        let api_group = unsafe { &mut *(group.api_group as *mut AfbGroup) };
        api_group.register(api.get_apiv4(), AFB_NO_AUTH);
        api.add_group(api_group);
        self
    }

    pub fn set_autorun(&'static mut self, value: bool) -> &'static mut Self {
        self.autorun = value;
        self
    }

    pub fn set_autoexit(&'static mut self, value: bool) -> &'static mut Self {
        self.autoexit = value;
        self
    }

    pub fn set_output(&'static mut self, value: AfbTapOutput) -> &'static mut Self {
        self.output = value;
        self
    }

    pub fn set_info(&'static mut self, value: &'static str) -> &'static mut Self {
        self.info = value;
        let api = unsafe { &mut *(self.tap_api as *mut AfbApi) };
        api.set_info(value);
        self
    }

    pub fn get_api(&self) -> &AfbApi {
        unsafe { &*(self.tap_api as *mut AfbApi) }
    }

    pub fn get_uid(&self) -> &'static str {
        self.uid
    }

    pub fn get_event(&self) -> &AfbEvent {
        unsafe { &*(self.event as *mut AfbEvent) }
    }

    pub fn get_group(&self, label: &'static str) -> Option<&AfbTapGroup> {
        let api = self.get_api();

        match self.hashmap.get(label) {
            Some(group) => {
                afb_log_msg!(Debug, api, "-- Get Tap group:{}", label);
                Some(unsafe { &mut *(*group as *mut AfbTapGroup) })
            }
            None => {
                afb_log_msg!(Critical, api, "Fail to find test-group:{}", label);
                None
            }
        }
    }
    #[track_caller]
    // launch a group and return report as jsonc
    pub fn launch(&self, label: &'static str) -> Result<(), AfbError> {
        match self.get_group(label) {
            None => afb_error!("group-label-not-found", label),
            Some(group) => group.launch(),
        }
    }
    #[track_caller]
    pub fn finalize(&'static mut self) -> Result<(), AfbError> {
        let api = unsafe { &mut *(self.tap_api as *mut AfbApi) };
        let vcbdata = TapGroupData {
            group: self.autostart,
        };

        // add auto start group verbs
        let autostart_tap = unsafe { &mut *(self.autostart as *mut AfbTapGroup) };
        let autostart_afb = unsafe { &mut *(autostart_tap.api_group as *mut AfbGroup) };
        autostart_afb.register(api.get_apiv4(), AFB_NO_AUTH);
        api.add_group(autostart_afb);

        let verb = AfbVerb::new(AUTOSTART);
        verb.set_callback(tap_group_callback)
            .set_context(vcbdata)
            .set_info("default tap autostart group")
            .set_usage("no_input")
            .register(api.get_apiv4(), AFB_NO_AUTH);
        api.add_verb(verb.finalize()?);

        // seal tap test api
        unsafe { cglue::afb_api_seal(api.get_apiv4()) };

        if self.autorun {
            match AfbSchedJob::new(self.uid)
                .set_callback(tap_suite_callback)
                .set_context(TapSuiteAutoRun { suite: self })
                .post(0, AFB_NO_DATA)
            {
                Err(error) => Err(error),
                Ok(_job) => Ok(()),
            }
        } else {
            Ok(())
        }
    }

    pub fn get_autoexit(&self) -> bool {
        self.autoexit
    }

    pub fn get_autorun(&self) -> bool {
        self.autorun
    }

    pub fn get_report(&'static mut self) -> Result<JsoncObj, AfbError> {
        let autostart = unsafe { &mut *(self.autostart) };
        let jreport = JsoncObj::new();
        jreport.add(AUTOSTART, autostart.get_report()?)?;

        for (uid, group) in self.hashmap.drain() {
            let group = unsafe { &mut (*group) };
            jreport.add(uid, group.get_report()?)?;
        }

        match self.output {
            AfbTapOutput::NONE => {}
            AfbTapOutput::JSON => {
                println!("{}", jreport);
            }
            AfbTapOutput::TAP => {
                println!("-- start:{} --", self.uid);
                let jvec = jreport.expand()?;
                for entry in &jvec {
                    let _key = entry.key.as_str();
                    println!();
                    match jreport.get::<JsoncObj>(entry.key.as_str()) {
                        Err(error) => {
                            afb_log_msg!(Critical, self.get_api().get_apiv4(), error.to_string());
                        }
                        Ok(jtest) => {
                            for idx in 0..jtest.count().unwrap() {
                                println!("{}", jtest.index::<String>(idx).unwrap().as_str());
                            }
                        }
                    }
                }
                println!("\n-- end:{} --", self.uid);
            }
        };
        Ok(jreport)
    }
}

struct TapSuiteAutoRun {
    suite: *mut AfbTapSuite,
}

fn tap_suite_callback(
    job: &AfbSchedJob,
    _signal: i32,
    _args: &AfbCtxData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let context = ctx.get_ref::<TapSuiteAutoRun>()?;
    let suite = unsafe { &mut *(context.suite as *mut AfbTapSuite) };
    let autostart = unsafe { &mut *(suite.autostart) };

    match autostart.launch() {
        Err(error) => {
            afb_log_raw!(
                Critical,
                suite.get_api().get_apiv4(),
                "Test fail {}:autostart error={}",
                suite.get_uid(),
                error
            );
        }
        Ok(()) => {}
    }

    let autoexit = suite.get_autoexit();
    suite.get_report()?;
    job.terminate();

    if autoexit {
        std::process::exit(0);
    }
    Ok(())
}

// implement TapTest API callback
struct TapTestData {
    test: *mut AfbTapTest,
}
#[track_caller]
fn tap_test_callback(
    rqt: &AfbRequest,
    _args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let context = ctx.get_ref::<TapTestData>()?;

    // bypass Rust limitation that refuses to understand static object pointers
    let test = unsafe { &mut (*context.test) };
    match test.jobpost() {
        Err(error) => {
            afb_log_msg!(Error, rqt, "fail to launch test error={}", error);
            rqt.reply(error, 405);
        }
        Ok(_jreport) => {
            // wait for test to be completed
            let _next = test.get_next();
            rqt.reply(test.get_report()?, 0);
        }
    }
    Ok(())
}

// implement TapGroup API callback
struct TapGroupData {
    group: *mut AfbTapGroup,
}

#[track_caller]
fn tap_group_callback(
    rqt: &AfbRequest,
    _args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let context = ctx.get_ref::<TapGroupData>()?;
    // bypass Rust limitation that refuses to understand static object pointers
    let group = unsafe { &mut (*context.group) };
    let suite = unsafe { &mut (*group.suite) };
    let event = unsafe { &mut (*suite.event) };

    event.subscribe(rqt)?;

    match group.launch() {
        Err(error) => {
            afb_log_msg!(Error, rqt, "fail to launch test error={}", error);
            rqt.reply(error, 405);
        }
        Ok(_jreport) => {
            rqt.reply(group.get_report()?, 0);
        }
    }
    Ok(())
}
