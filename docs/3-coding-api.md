# Coding binding APIs

## Beginning your first rust binding file

At first your file should start with code:

```rust
// import libafb dependencies
extern crate afbv4;

// import libafb dependencies
use afbv4::prelude::*;
```

This code will allow you to use rust binding code in your program.

## Exposing api

Rust is not very friendly with static mutable variables also contrarily to C/C++ versions, afb-librust exposes an 'object' centric API where all static data are hidden from the developer.

Rust-libafb binding main entry point is implemented trough ```AfbBindingRegister!(my_init_callback)``` that fake C/C++ afb-v4 binding entry points.
At binding loading time afb-binder will call ```my_init_callback``` passing api config as jsonc object as input parameter.

**Note: that by default any created API received two builtin verbs.**

* ```api/ping``` typically use to check if binding/api is alive or not.
* ```api/info``` api introspection verb used with debug and monitoring.

```rust
// check examples/demo/demo-binding.rs for full code
// Binding init callback started at binding load time before any API exist
// -----------------------------------------
pub fn binding_init(rootv4: AfbApiV4, jconf: AfbJsonObj) -> Result <&'static AfbApi, AfbError> {
    afb_log_msg!(Notice, rootv4, "-- binding-init binding config={}", jconf);

    #[allow(deref_nullptr)] // event and timer are updated at api init time
    if status == 0 {
        status = match AfbApi::new("rust-api")
            .set_name("rust-api")
            .set_info("My first Rust API")
            .set_permission(AfbPermission::new("acl:rust"))
            .add_verb(verb_basic::register(binding))
            .add_verb(verb_typed::register(binding))
            .set_callback(Box::new(ApiUserData {}))
            .finalize()
        {
            Ok(api) => {
                afb_log_msg!(Notice, binding, "RUST api uid={} started", api.get_uid());
                AFB_OK
            }
            Err(error) => {
                afb_log_msg!(Critical, binding, "Fail to register api error={}", error);
                AFB_ABORT
            }
        }
    };
    status
}

// register binding within libafb
AfbBindingRegister!(binding_init);
```

After AfbApi::finalization() registering the newly created API, libafb framework optionally calls user defined
callbacks implementing AfbApiControls trait.

```rust
pub struct ApiUserData {
    _any_data: &'static str,
}

// AfbApi userdata should implement AfbApiControls trait
// trait provides default callback for: config,ready,orphan,class,exit
impl AfbApiControls for ApiUserData {
    // api is loaded but not ready to be used, when defined binder send binding specific configuration
    fn config(&mut self, api: &AfbApi, config: AfbJsonObj) -> Result<(),AfbError> {
        let _api_data = self; // self matches api_data
        afb_log_msg!(
            Notice,
            api,
            "--api-config api={} config={}",
            api.get_uid(),
            config
        );

        Ok(())
    }

    // the API is created and ready. At this level user may subcall api(s) declare as dependencies
    fn start(&mut self, _api: &AfbApi) ->  Result<(),AfbError> {
        let _api_data = self; // self matches api_data
        Ok(())
    }

    // mandatory for downcasting back to custom apidata object
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
```

## Exposing verbs

Except in special case as test, all API exposed some user-defined verbs. While lib-afb framework only support a flat hierarchy of api/verb, afb-librust permits to group verb when they share a common prefix or access control privilege/loa.

* Verb optionally carry a static private userdata call context. When defined verb callback receive an extra userdata parameter.
* Verb systematically receive a request, this request is used as a global libafb handle within the callback
* Verb optionally receive parameters. Those parameters are typed with builtin libafb/types (i.e. int,jsonc,string,...) or respond
      to a custom user defined type as ```MySimpleType``` in following sample. Note that custom encoding/decoding function are
      implemented automatically.

To create a verb developer should:

* link verb callback ```.set_callback(my_callback)```
* optionally link verb with userdata ```.set_context (myStruct {init values})```
* implement api logic ```callback(request, params, vcbdata)```
* create verb object with ```AfbVerb::new("verb-uid")```
* optionally register custom user-defined data type ```AfbDataConverter!() + simple_data::register()```
* finally add the verb to the API or to a group with ```api.ad_verb(handle{})```

***Warning: when defined user vcbdata struct/type should be unique to module namespace***

```rust
// extract from examples/demo/demo-verb*.rs
AfbDataConverter!(simple_data, MySimpleType);
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MySimpleType {
    pub name: String,
    pub x: i32,
    pub y: i32,
}

struct MySimpleContext {
    x: i32,
    ....
}

fn simple_callback(request: &AfbRequest, args: &mut AfbRqtData, ctx: AfbCtxData) -> Result<(),AfbError> {
    // check arg0 match MySimpleType grammar
    let arg0 = args.get::<&MySimpleType>(0)?;

    // get use context data
    let usrdata= ctx.get_ref::<MySimpleContext>()?;
    let usrdata.x += 1;

    // we have a valid simple input data
    afb_log_msg!(Notice, request, "got simple-data={:?}", input);
    // create a sample simple-data object as response
    let output = MySimpleType {
        name: input.name.to_uppercase(),
        x: input.x + 1,
        y: input.y - 1,
    };
    // closure is call from following 'if let' with reply()
    let reply = || -> Result<(), AfbError> {
        let mut response = AfbParams::new();
        response.push(output)?;
        request.reply(response, 0);
        Ok(())
    };
    // if data export fail send an error report
    if let Err(error) = reply() {
        request.reply(afb_add_trace!(error), 405);
    }
    Ok(())
}

// register user custom defined data type within libafb framework
match simple_data::register() {
    Err(error) => {
        afb_log_msg!(Critical,binding,"fail to register converter error={}",error);
        panic! ("(hoops) fail to register custom type");
    }
    Ok(_value) => {},
};

// register verb handle within API
AfbVerb::new("my-simple-verb")
        .set_callback(simple_callback)
        .set_info("My custom type demo verb")
        .set_usage("any json string")
        .set_sample("{'lander': 'Brittany', 'location':'Europe'}")
        .expect("invalid json sample")
        .finalize()
```

## API/RQT Subcalls

Both synchronous and asynchronous call are supported. The fact the subcall is done from a request or an api context is abstracted to the user; both model share the same method signature. When doing it from RQT context client security context is not propagated and remove event are claimed by the rust api.

Explicit response to a request is done with ```request.reply(data, status)```. When returning more than one arguments, those one should be packed with ```AfbParams::new().push()...```

```rust
// async response callback uses standard (AfbVerbRegister!) callback
fn async_response_cb(request: &AfbRequest, params: &mut AfbRqtData, ctx: AfbCtxData) -> Result<(),AfbError> {
    let jquery = match params.get::<JsoncObj>(0) ?;
    let usrdata= ctx.get_ref::<MyCtxType>()?;

    // do something
    request.reply("async callback done", 0);
    Ok(())
}

// Depending on execution context, handle is either a request or an api.
// ----------------------------------------------------------------------

// asynchronous subcall
match AfbSubCall::call_async(handle,"api-test","ping",AFB_NO_DATA, async_response_cb, MyCtxType{}) {
    Err(error) => {
        afb_log_msg!(Error, &handle, &error);
        handle.reply(afb_add_trace!(error), -1)
    }
    Ok(()) => {afb_log_msg!(Notice, handle, "async call handle accepted");}
};

// synchronous subcall
match AfbSubCall::call_sync(handle, "api-test", "ping", AFB_NO_DATA) {
    Err(error) => {
        afb_log_msg!(Error, handle, &error);
        handle.reply(afb_add_trace!(error), -1)
    }
    Ok(mut response) => {
        let status = response.get_status();
        afb_log_msg!(handle, "call processed response={} status={}", response, status);
    }
};
```

## Events

Events can be split in two classes:

* events you receive
* events you sent

Receiving events is very similar to a standard api/verb request callback. Received event is a special form of unattended request where:

* no response to the sender is possible.
* no permission or authentication is possible
* no session is defined
* request handle is replace with an event handle
* event callback uses ```.set_callback()``` optionally ```.set_context()```
* you subscribe to an event pattern in place of an api/verb
* input arguments and userdata are the same as for api/verb
* event should be register in the API either directly or thought a group

```rust
// event callback is similar to verb callback expect for AfbEventRegister!
fn event_get_callback(event: &AfbEventMsg, args: &mut AfbRqtData, ctx: &AfbCtxData) -> Result<(),AfbError> {
    let userdata = ctx.get_ref::<EvtUserData>()?;
    userdata.counter += 1;
    afb_log_msg!(Notice,&event,"evt={} name={} counter={}",event.get_uid(),event.get_name(), userdata.counter);
    let argument= match args.get::<JsoncObj>(0)?;
    Ok(())
}

// create the event object with its pattern and optionally userdata
let event_handler = AfbEvtHandler::new("handler-1")
    .set_info("My first event handler")
    .set_pattern("helloworld-event/timerCount")
    .set_callback(event_get_callback)
    .set_context(EvtUserData { counter: 0 })
    .finalize();

// register event within the API directly or through a group
let group=  AfbGroup::new("event")
        .set_info("event demo group")
        .add_evt_handler(event_handler)
        .finalize()
```

When sending event you have to:

* create the event
* subscribe/unsubscribe client to event from the request

Afb Events require to be attached to an API. As a result they are typically created from API control callback at init time when API reaches 'ready' state. Note that it is developer responsibility to make AfbEvent handle visible from both the function that create the event to the function that use the event. As Rust is not friendly with mutable static, the best place to keep track of event handle is either the Api userdata or Verb VcbData when a unique verb is use for both subscribe+unsubscribe.

Note than when event handle is store within API user data, it is the responsibility of the developer to downcast from generic AfbApiControls trait to custom Api user data as defined by developer.

Note: to keep following example as simple as possible (subscribe,unsubscribe,push) actions are implemented in 3 different verbs. Which is very rare. Most of the time event are generated because some data need to be publish, and event push is attached to reading a device, a timer and anything that produce data asynchronously from any request.

```rust
use ApiUserData;

fn subscribe_callback(request: &AfbRequest, _args: &mut AfbRqtData, _ctx: AfbCtxData) -> Result<(),AfbError> {
    let apidata = request
        .get_apidata()
        .downcast_ref::<ApiUserData>()
        .expect("invalid api-data");

    match apidata.my_event.subscribe(request) {
        Err(error) => request.reply(afb_add_trace!(error), 405),
        Ok(_event) => request.reply(AFB_NO_DATA, 0),
    }
    Ok(())
}

fn unsubscribe_callback(request: &AfbRequest, _args: &mut AfbRqtData, _ctx: AfbCtxData) -> Result<(),AfbError> {
    let apidata = request
        .get_apidata()
        .downcast_ref::<ApiUserData>()
        .expect("invalid api-data");

    match apidata.my_event.unsubscribe(request) {
        Err(error) => request.reply(afb_add_trace!(error), 405),
        Ok(_event) => request.reply(AFB_NO_DATA, 0),
    }
    Ok(())
}

fn push_callback(request: &AfbRequest, args: &mut AfbRqtData, _ctx: AfbCtxData) -> Result<(),AfbError> {
    let apidata = request
        .get_apidata()
        .downcast_ref::<ApiUserData>()
        .expect("invalid api-data");

    let jquery = match args.get::<JsoncObj>(0)?;
    // increment event counter and push event to listener(s)
    let listeners = apidata.my_event.push(jquery);
    request.reply(format!("event listener listeners={}", listeners), 0);
    Ok(())
}

// event depends on API and should be create only after api is ready
pub fn start(api: &AfbApi, api_data: &mut ApiUserData) -> Result<(), AfbError> {
    // store event handle within API userdata to make it visible from any verb
    match AfbEvent::new(api, "my-event") {
        Err(error) => {
            afb_log_msg!(Critical, api, &error);
            Err(error)
        }
        Ok(event) => {
            api_data.my_event = event;
            Ok(())
        }
    }
    Ok(())
}
```

## Timers

Timer are typically used to push event or to handle timeout. LibAfb supports two classes of timers:

* full timer, that leverage linux kernel timerfd capabilities
* delay timer, that rely on libafb job posting

Timer callback are very similar to api/verb callback except that they require ```AfbTimerRegister!()``` and receive a timer
handle in place of a request handle.

Full timer takes:

* period: delay in ms that defines callback tic rate
* count: the number of time, the timer should tic (default: zero== infinite)

Note: timer handle is allocate in heap and deleted only when decount reach zero. If timer.set_count(0) it runs until afb_binder exit. User does not have to care about the timer handle live cycle as AfbTimer::new leak the handle memory.

```rust
// Use timer vcbdata to store event handle that is normally available from API userdata
struct TimerUserData {
    counter: u32,
    event: &'static AfbEvent,
}

// Callback is called for each tick until decount>0
fn timer_callback(timer: &AfbTimer, decount: i32, ctx: &mut AfbCtxData) -> Result<(),AfbError> {
    let usedata= ctx.get_ref::<TimerUserData>()?;
    userdata.counter += 1;

    afb_log_msg!(Notice,timer,"timer={} counter={} decount={}", timer.get_uid(),userdata.counter,decount);
    let _count = userdata.event.push(userdata.counter);
    Ok(())
}

let timer= match AfbTimer::new("demo_timer")
    .set_period(3000)
    .set_count(10)
    .set_callback(timer_callback)
    .set_context(TimerUserData {
        counter: 0,
        event: apidata.my_event,
    })
    .start()
    {
        Err(error) => {
            afb_log_msg!(Critical, request, &error);
            request.reply(afb_add_trace!(error), -1)
        }
        Ok(timer) => {
            apidata.my_timer= timer;
            request.reply("timer started",0);
        }
    }
```

Delay timers are simpler and run only once. They take two params

* delay: time in ms before raising the callback
* watchdog: maximum time in second to run the callback

At delay the callback is activated with signal==0. Is ever callback runs longer that watchdog callback is activated again with a signal value!=0.

```rust
// Job verb data is valid during all verb live cycle
struct JobVerbContext {
    job_handle: &'static AfbSchedJob
}

struct JobPostData {
    rqt: AfbRqtV4,
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
    let params = args.get::<JobPostData>()?;
    let request = AfbRequest::from_raw(params.rqt);

    let mut response = AfbParams::new();
    response.push(&params.jsonc)?;
    request.reply(response, signal);
    Ok(())
}

let event = AfbEvent::new("demo-job-event");

// create a new job handle
let job_handle= AfbSchedJob::new("demo-job-delay")
        .set_exec_watchdog(10) // limit exec time to 1s;
        .set_callback(jobpost_callback)
        .finalize();

let job_verb = AfbSchedJob::new("demo-job-post")
        .set_exec_watchdog(10) // limit exec time to 10s;
        .set_group(1)
        .set_callback(jobpost_callback)
        .set_context(JobStaticContext {job_handle})
        .finalize();

// register verb within api
```
