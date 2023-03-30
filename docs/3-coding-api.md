# Coding binding APIs

## Exposing api

Rust is not very friendly with static mutable variables also contrarily to C/CC+ versions, afb-librust exposes an 'object' centric API where all static data are hidden from the developer.

Rust-libafb binding main entry point is implemented trough ```AfbBindingRegister!(my_init_callback)``` that fake C/C++ afb-v4 binding entry points.
At binding loading time afb-binder will call ```my_init_callback``` passing api config as jsonc object as input parameter.

**Note: that by default any created API received two builtin verbs.**

    * ```api/ping``` typically use to check if binding/api is alive or not.
    * ```api/info``` api introspection verb used with debug and monitoring.

```rust
// check examples/demo/demain.rs for full code
pub fn binding_init(binding: AfbApiV4, jconf: Jsonc) -> i32 {
    afb_log_msg!(Notice, binding, "-- binding-init binding config={}", jconf);
    let mut status = 0;

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
                AFB_FATAL
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
    my_event: &'static AfbEvent,
    my_timer: &'static mut dyn AfbTimerRef,
}
impl AfbApiControls for ApiUserData {
    // api is loaded but not ready to be used, when defined binder send binding specific configuration
    fn config(&mut self, api: &AfbApi, config: Jsonc) -> i32 {
        let _api_data = self; // self matches api_data
        afb_log_msg!(Notice, api, "--api-config api={} config={}", api.get_uid(), config);
        AFB_OK // returning -1 will abort binder process
    }

    // the API is created and ready. At this level user may create event, timers, declare dependencies,...
    fn start(&mut self, api: &AfbApi) -> i32 {
        let api_data = self; // self matches api_data
        // create event and store its handle at api userdata level
        match event_group::start(api, api_data) {
            Err(_error) => {return AFB_FAIL;},
            Ok(()) => {}
        };
        AFB_OK
    }
}
```

## Exposing verbs

Except in special case as test, all API exposed some user-defined verbs. While lib-afb framework only support a flat hierarchy of api/verb, afb-librust permits to group verb when they share a common prefix or access control privilege/loa.

    * Verb optionally carry a static private userdata call vcbdata. When defined verb callback receive an extra userdata parameter.
    * Verb systematically receive a request, this request is used as a global libafb handle within the callback
    * Verb optionally receive parameters. Those parameters are typed with builtin libafb/types (i.e. int,jsonc,string,...) or respond
      to a custom user defined type as ```MySimpleData``` in following sample. Note that custom encoding/decoding function are
      implemented automatically.

To create a verb developer should:
    * link verb callback and userdata ```AfbVerbRegister!(handle, callback, [vcbdata])```
    * implement api logic ```callback(request, params, [vcbdata])```
    * create verb object with ```AfbVerb::new("verb-uid")```
    * optionally register custom user-defined data type ```AfbDataConverter!() + simple_data::register()```
    * finally add the verb to the API or to a group with ```api.ad_verb(handle{})```
*** Warning: when defined user vcbdata struct/type should be unique to module namespace ***

```rust
// extract from examples/demo/demo-verb*.rs
AfbDataConverter!(simple_data, MySimpleData);
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Default)]
struct MySimpleData {name: String,x: i32,y: i32,}

AfbVerbRegister!(VerbCtrl, callback);
fn callback(request: &AfbRequest, args: &mut AfbData) {
    // check arg0 match MySimpleData grammar
    let arg0 = args.get::<&'static MySimpleData>(0);
    let input = match arg0 {
        Err(mut error) => {
            afb_log_msg!(Error, request, "invalid args[0] error={}", error);
            request.reply(afb_add_trace!(error), 405);
            return;
        }
        Ok(data) => data,
    };
    // we have a valid simple input data
    afb_log_msg!(Notice, request, "got simple-data={:?}", input);
    // create a sample simple-data object as response
    let output = MySimpleData {
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
    if let Err(mut error) = reply() {
        request.reply(afb_add_trace!(error), 405);
    }
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
AfbVerb::new("my-verb")
        .set_callback(Box::new(VerbCtrl {}))
        .set_info("My custom type demo verb")
        .set_usage("any json string")
        .set_sample("{'lander': 'Brittany', 'location':'Europe'}")
        .expect("invalid json sample")
        .finalize()
```

## API/RQT Subcalls

Both synchronous and asynchronous call are supported. The fact the subcall is done from a request or an api context is abstracted to the user; both model share the same method signature. When doing it from RQT context client security context is not propagated and remove event are claimed by the rust api.

Explicit response to a request is done with ``` request.reply(data, status)```. When returning more than one arguments, those one should be packed with ```AfbParams::new().push()...```

```rust
// async response callback uses standard (AfbVerbRegister!) callback
AfbVerbRegister!(AsyncResponseCtrl, async_response_cb);
fn async_response_cb(request: &AfbRequest, params: &mut AfbData) {
    let jquery = match params.get::<AfbJsonObj>(0) {
        Ok(argument) => {afb_log_msg!(Info,request,"async_response received params={}",argument);}
        Err(mut error) => {
            afb_log_msg!(Error, request, "async_response error={}", error);
            request.reply(afb_add_trace!(error), -1);
            return;
        }
    };
    // do something
    request.reply("async callback done", 0);
}

// Depending on execution context, handle is either a request or an api.
// ----------------------------------------------------------------------

// asynchronous subcall
match AfbSubCall::call_async(handle,"api-test","ping",AFB_NO_DATA, Box::new(AsyncResponseCtrl{})) {
    Err(error) => {
        afb_log_msg!(Error, &handle, &error);
        handle.reply(afb_add_trace!(error), -1)
    }
    Ok(()) => {afb_log_msg!(Notice, handle, "async call handle accepted");}
};

// synchronous subcall
match AfbSubCall::call_sync(handle, "api-test", "ping", AFB_NO_DATA) {
    Err(mut error) => {
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
  * event callback uses ```AfbEventRegister!```
  * you subscribe to an event pattern in place of an api/verb
  * input arguments and userdata are the same as for api/verb
  * event should be register in the API either directly or thought a group

```rust
// event callback is similar to verb callback expect for AfbEventRegister!
AfbEventRegister!(EventGetCtrl, event_get_callback, EvtUserData);
fn event_get_callback(event: &AfbEventMsg, args: &mut AfbData, userdata: &mut EvtUserData) {
    userdata.counter += 1;
    afb_log_msg!(Notice,&event,"evt={} name={} counter={}",event.get_uid(),event.get_name(), userdata.counter);
    match args.get::<AfbJsonObj>(0) {
        Ok(argument) => {
            afb_log_msg!(Info, event, "Got valid jsonc object argument={}", argument);
            argument
        }
        Err(error) => {
            afb_log_msg!(Error, event, "hoop invalid json argument {}", error);
            AfbJsonObj::from("invalid json input argument")
        }
    };
}

// create the event object with its pattern and optionally userdata
let event_handler = AfbEvtHandler::new("handler-1")
    .set_info("My first event handler")
    .set_pattern("helloworld-event/timerCount")
    .set_callback(Box::new(EvtUserData { counter: 0 }))
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

AfbVerbRegister!(SubscribeCtrl, subscribe_callback);
fn subscribe_callback(request: &AfbRequest, _args: &mut AfbData) {
    let apidata = request
        .get_apidata()
        .downcast_ref::<ApiUserData>()
        .expect("invalid api-data");

    match apidata.my_event.subscribe(request) {
        Err(mut error) => request.reply(afb_add_trace!(error), 405),
        Ok(_event) => request.reply(AFB_NO_DATA, 0),
    }
}

AfbVerbRegister!(UnsubscribeCtrl, unsubscribe_callback);
fn unsubscribe_callback(request: &AfbRequest, _args: &mut AfbData) {
    let apidata = request
        .get_apidata()
        .downcast_ref::<ApiUserData>()
        .expect("invalid api-data");

    match apidata.my_event.unsubscribe(request) {
        Err(mut error) => request.reply(afb_add_trace!(error), 405),
        Ok(_event) => request.reply(AFB_NO_DATA, 0),
    }
}

AfbVerbRegister!(PushCtrl, push_callback);
fn push_callback(request: &AfbRequest, args: &mut AfbData) {
    let apidata = request
        .get_apidata()
        .downcast_ref::<ApiUserData>()
        .expect("invalid api-data");

    let jquery = match args.get::<AfbJsonObj>(0) {
        Ok(argument) => argument,
        Err(error) => {
            afb_log_msg!(Error, request, "hoop invalid json argument {}", error);
            AfbJsonObj::from("no-data")
        }
    };

    // increment event counter and push event to listener(s)
    let listeners = apidata.my_event.push(jquery);
    request.reply(format!("event listener listeners={}", listeners), 0);
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
AfbTimerRegister!(TimerCtrl, timer_callback, TimerUserData);
fn timer_callback(timer: &AfbTimer, decount: i32, userdata: &mut TimerUserData) {
    userdata.counter += 1;

    afb_log_msg!(Notice,timer,"timer={} counter={} decount={}", timer.get_uid(),userdata.counter,decount);
    let _count = userdata.event.push(userdata.counter);
}

let timer= match AfbTimer::new("demo_timer")
    .set_period(3000)
    .set_count(10)
    .set_callback(Box::new(TimerUserData {
        counter: 0,
        event: apidata.my_event,
    }))
    .start()
    {
        Err(mut error) => {
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
// delay userdata is used to keep track of the request
struct DelayCtxData {rqt: AfbRqtV4}

// this callback starts from AfbSchedJob::new. If signal!=0 then callback overpass its watchdog timeout
AfbJobRegister!(DelayCtrl, jobpost_callback, DelayCtxData);
fn jobpost_callback(job: &AfbSchedJob, signal: i32, userdata: &mut DelayCtxData) {
    let mut request = AfbRequest::from_raw(userdata.rqt);
    request.reply(userdata.jsonc.clone(), 300);
}

// post a job at 3s with a clone of the received json query
AfbVerbRegister!(JobPostVerb, jobpost_verb);
fn jobpost_verb(request: &AfbRequest, args: &mut AfbData) {

    match AfbSchedJob::new("demo-delay")
        .set_exec_watchdog(10) // limit exec time to 1s;
        .set_callback(Box::new(DelayCtxData {rqt: request.add_ref()}))
        .post(3000) // respond to request in 3s
    {
        Err(mut error) => request.reply(afb_add_trace!(error), -1),
        _ => {}
    }
}
```