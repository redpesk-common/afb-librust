/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk samples code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 */

// import libafb dependencies
use afbv4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

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
    ctx: Rc<UserCtxData>,
}

// Callback is called for each tick until decount>0
fn timer_callback(timer: &AfbTimer, decount: u32, ctx: &AfbCtxData) -> Result<(), AfbError> {
    // check request introspection
    let timer_uid = timer.get_uid();
    let context = ctx.get_ref::<UserVcbData>()?;
    let count = context.ctx.incr_counter();

    afb_log_msg!(
        Notice,
        timer,
        "--callback timer={} counter={} decount={}",
        timer_uid,
        count,
        decount
    );
    let _count = context.ctx.event.push(context.ctx.get_counter());

    ctx.free::<UserVcbData>();
    Ok(())
}

fn start_timer_callback(
    request: &AfbRequest,
    _args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    // subscribe client to event
    let context = ctx.get_ref::<UserVcbData>()?;
    context.ctx.event.subscribe(request).unwrap();

    // timer get require private instantiation of TimerUserData
    AfbTimer::new("demo_timer")
        .set_period(1000)
        .set_decount(10)
        .set_callback(timer_callback)
        .set_context(UserVcbData {
            ctx: context.ctx.clone(),
        })
        .start()?;

    request.reply(AFB_NO_DATA, 0);
    Ok(())
}

struct JobPostData {
    rqt: AfbRequest,
    jsonc: JsoncObj,
    count: u32,
}

// this callback starts from AfbSchedJob::new. If signal!=0 then callback overpass its watchdog timeout
fn jobpost_callback(
    job: &AfbSchedJob,
    signal: i32,
    args: &AfbCtxData,
    _ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    // retrieve job post arguments
    let params = args.get_ref::<JobPostData>()?;
    let request = &params.rqt;
    afb_log_msg!(
        Info,
        job,
        "{}: jobpost callback signal={} jsonc:{} count={}",
        job.get_uid(),
        signal,
        params.jsonc,
        params.count
    );
    let mut response = AfbParams::new();

    response.push(&params.jsonc)?;
    request.reply(response, signal);
    args.free::<JobPostData>();
    Ok(())
}

// post a job at 3s with a clone of the received json query
struct UserPostVerb {
    event: &'static AfbEvent,
    job_post: &'static AfbSchedJob,
    count: u32,
}
fn jobpost_verb(request: &AfbRequest, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    // extract jquery from 1st argument
    let context = ctx.get_mut::<UserPostVerb>()?;
    let jquery = match args.get::<JsoncObj>(0) {
        Ok(argument) => argument,
        Err(error) => error.to_jsonc()?,
    };

    // increase count
    context.count += 1;

    let job_context = JobPostData {
        rqt: request.add_ref(),
        jsonc: jquery.clone(),
        count: context.count,
    };

    let _jobid = context.job_post.post(3000, job_context)?;

    match context.event.subscribe(request) {
        Err(_error) => {}
        Ok(event) => {
            event.push(format!(
                "job-post response should arrive in 3s count={}",
                context.count
            ));
        }
    }
    Ok(())
}

// prefix group of event verbs and attach a default privilege
pub fn register(apiv4: AfbApiV4) -> Result<&'static AfbGroup, AfbError> {
    // build verb name from Rust module name
    let mod_name = module_path!().split(':').last().unwrap();
    afb_log_msg!(Notice, apiv4, "Registering group={}", mod_name);

    let event = AfbEvent::new("timer-event").finalize()?;
    let ctxdata = Rc::new(UserCtxData {
        counter: Cell::new(0),
        event: event,
    });

    let start_timer = AfbVerb::new("timer-start")
        .set_callback(start_timer_callback)
        .set_context(UserVcbData {
            ctx: ctxdata.clone(),
        })
        .set_info("tics 1s timer for 10 tic")
        .set_usage("no input")
        .finalize()?;

    let job_post = AfbSchedJob::new("demo-job-post-verb-cb")
        .set_exec_watchdog(10) // limit exec time to 10s;
        .set_group(1)
        .set_callback(jobpost_callback)
        .finalize();

    let job_verb = AfbVerb::new("job-post")
        .set_callback(jobpost_verb)
        .set_context(UserPostVerb {
            event,
            job_post,
            count: 0,
        })
        .set_info("return response in 3s")
        .set_usage("no input")
        .finalize()?;

    let group = AfbGroup::new(mod_name)
        .set_info("timer demo api group")
        .set_prefix(mod_name)
        .set_permission(AfbPermission::new("acl:evt"))
        .set_verbosity(3)?
        .add_verb(start_timer)
        .add_verb(job_verb)
        .add_event(event)
        .finalize()?;

    Ok(group)
}
