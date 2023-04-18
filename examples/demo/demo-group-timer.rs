/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk samples code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 */

// import libafb dependencies
use libafb::prelude::*;
use std::cell::Cell;
use std::sync::Arc;

//lib afb support two class of timers
// - schedjob is delay+watchdog it starts a callback after a delay(ms) and limit exec time(seconds).
//   Note: schedjob depends on libafb thread pool. It is a very lightweight mechanisms with limited precision.
// - AfbTimer relies on Linux timer-fd, it requires more config but better leverage of kernel capabilities.
// event callback require to share event handle.
// - nevertheless Vcbdata even when using the same type remain private to individual verb
// - verb are created at config time and event only after API is ready (start time)
// - as event handle should be attach to api userdata let's import it


struct UserCtxData {
    event: &'static AfbEvent,
    counter: Cell<u32>,
}

impl UserCtxData {
    fn incr_counter(&self) -> u32 {
        self.counter.set(self.counter.get() + 1);
        self.counter.get()
    }

    fn get_counter(&self) -> u32 {
        self.counter.get()
    }
}

// Use timer vcbdata to store event handle that is normally available from API userdata
struct UserVcbData {
    ctx: Arc<UserCtxData>,
}

// Callback is called for each tick until decount>0
AfbTimerRegister!(TimerCtrl, timer_callback, UserVcbData);
fn timer_callback(timer: &AfbTimer, decount: u32, userdata: &mut UserVcbData) {
    // check request introspection
    let timer_uid = timer.get_uid();
    let count = userdata.ctx.incr_counter();

    afb_log_msg!(
        Notice,
        timer,
        "--callback timer={} counter={} decount={}",
        timer_uid,
        count,
        decount
    );
    let _count = userdata.ctx.event.push(userdata.ctx.get_counter());
}

AfbVerbRegister!(StartTimerCtrl, start_timer_callback, UserVcbData);
fn start_timer_callback(request: &AfbRequest, _args: &AfbData, userdata: &mut UserVcbData) {

    // subscribe client to event
    userdata.ctx.event.subscribe(request).unwrap();

    // timer get require private instantiation of TimerUserData
    match AfbTimer::new("demo_timer")
        .set_period(1000)
        .set_decount(10)
        .set_callback(Box::new(UserVcbData {
            ctx: userdata.ctx.clone(),
        }))
        .start()
    {
        Err(mut error) => {
            afb_log_msg!(Critical, request, &error);
            request.reply(afb_add_trace!(error), -1);
        }
        Ok(_timer) => {
            request.reply("demp_timer started", 0);
        }
    }
}

struct UserPostData {
    rqt: AfbRqtV4,
    jsonc: AfbJsonObj,
}
// this callback starts from AfbSchedJob::new. If signal!=0 then callback overpass its watchdog timeout
AfbJobRegister!(DelayCtrl, jobpost_callback, UserPostData);
fn jobpost_callback(job: &AfbSchedJob, signal: i32, userdata: &mut UserPostData) {
    let request = AfbRequest::from_raw(userdata.rqt);
    afb_log_msg!(Info, job, "{}: jobpost callback signal={}", job.get_uid(), signal);
    request.reply(userdata.jsonc.clone(), signal);
}

// post a job at 3s with a clone of the received json query
struct UserPostVerb {
    event: &'static AfbEvent,
}
AfbVerbRegister!(JobPostVerb, jobpost_verb, UserPostVerb);
fn jobpost_verb(request: &AfbRequest, args: &AfbData, userdata: &mut UserPostVerb) {

    // extract jquery from 1st argument
    let jquery = match args.get::<AfbJsonObj>(0) {
        Ok(argument) => argument,
        Err(error) => error.to_jsonc(),
    };

    match AfbSchedJob::new("demo-job-post-verb-cb")
        .set_exec_watchdog(10) // limit exec time to 10s;
        .set_callback(Box::new(UserPostData {
            rqt: request.add_ref(),
            jsonc: jquery.clone(),
        }))
        .post(3000)
    {
        // exec job in ~3s
        Err(mut error) => {request.reply(afb_add_trace!(error), -1);},
        Ok(job) => {afb_log_msg!(Info, request, "Job posted uid:{} jobid={}", job.get_uid(), job.get_jobid());},
    }

    match userdata.event.subscribe(request) {
        Err(_error) => {},
        Ok(event) => {event.push("job-post response should arrive in 3s");},
    }

}

// prefix group of event verbs and attach a default privilege
pub fn register(apiv4: AfbApiV4) -> Result<&'static AfbGroup, AfbError> {
    // build verb name from Rust module name
    let mod_name = module_path!().split(':').last().unwrap();
    afb_log_msg!(Notice, apiv4, "Registering group={}", mod_name);

    let event = AfbEvent::new("timer-event").finalize()?;
    let ctxdata= Arc::new(UserCtxData {
        counter: Cell::new(0),
        event: event,
    });

    let start_timer = AfbVerb::new("timer-start")
        .set_callback(Box::new(UserVcbData {
            ctx: ctxdata.clone()
        }))
        .set_info("tics 1s timer for 10 tic")
        .set_usage("no input")
        .finalize()?;

    let job_post = AfbVerb::new("job-post")
        .set_callback(Box::new(UserPostVerb {
            event: event
        }))
        .set_info("return response in 3s")
        .set_usage("no input")
        .finalize()?;

    let group=AfbGroup::new(mod_name)
        .set_info("timer demo api group")
        .set_prefix(mod_name)
        .set_permission(AfbPermission::new("acl:evt"))
        .set_verbosity(3)
        .add_verb(start_timer)?
        .add_verb(job_post)?
        .add_event(event)?
        .finalize()?;

    Ok(group)
}
