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

// (subscribe,unsubscribe,push) using AfbVerbRegister! callbacks register or push event
// (event_get_callback) using AfbEventRegister! is called when corresponding event pattern is received

// event callback require to share event handle.
// - nevertheless Vcbdata even when using the same type remain private to individual verb
// - event handle is shared through a shared cell
// - an alternative would be to store event at api userdata level

// this sample also demonstrate request session. 'SessionUserData' is a private object that is
// available for each request and share in between all verbs until they are connected to the
// the same client/session.

struct UserCtxData {
    event: &'static AfbEvent,
    counter: Cell<u32>,
}

/// each verb share a common data type nevertheless as each verb get its own implementation
/// it is necessary to add an extra share structure with Rc/Arc to effectively share event/counter

impl UserCtxData {
    fn incr_counter(&self) -> u32 {
        self.counter.set(self.counter.get() + 1);
        self.counter.get()
    }
}

// impl SessionUserData {
//     fn get<'a>(request: &'a AfbRequest) -> Result<&'a mut Self, AfbError> {
//         match request.get_session() {
//             Err(error) => Err(error),
//             Ok(any) => match any.as_any().downcast_mut::<SessionUserData>() {
//                 None => Err(AfbError::make(
//                     "session-any-cast",
//                     "fail to restore <SessionUserData>",
//                 )),
//                 Some(value) => Ok(value),
//             },
//         }
//     }

//     fn set<'a>(request: &'a AfbRequest, userdata: SessionUserData) -> Result<&'a mut Self, AfbError> {
//         match request.set_session(Box::new(userdata)) {
//             Err(error) => Err(error),
//             Ok(any) => match any.as_any().downcast_mut::<SessionUserData>() {
//                 None => Err(AfbError::make(
//                     "session-any-cast",
//                     "fail to restore <SessionUserData>",
//                 )),
//                 Some(value) => Ok(value),
//             },
//         }
//     }

//     fn drop(request: &AfbRequest) -> Result<(), AfbError> {
//         request.drop_session()
//     }
// }

// impl AfbRqtSession for SessionUserData {
//     fn as_any(&mut self) -> &mut dyn Any {
//         self
//     }
// }


// attach to session (one per client)
AfbSessionRegister!(SessionUserData);
struct SessionUserData {
    count: u32,
}


struct SubscribeData {
    ctx: Arc<UserCtxData>,
}
AfbVerbRegister!(SubscribeCtrl, subscribe_callback, SubscribeData);
fn subscribe_callback(request: &AfbRequest, _args: &AfbData, userdata: &mut SubscribeData) {
    match SessionUserData::set(request, SessionUserData{count:0}) {
        Err(mut error) => return request.reply(afb_add_trace!(error), 405),
        Ok(value) => value
    };

    match userdata.ctx.event.subscribe(request) {
        Err(mut error) => request.reply(afb_add_trace!(error), 405),
        Ok(_event) => request.reply(AFB_NO_DATA, 0),
    }
}

struct UnsubscribeData {
    ctx: Arc<UserCtxData>,
}
AfbVerbRegister!(UnsubscribeCtrl, unsubscribe_callback, UnsubscribeData);
fn unsubscribe_callback(request: &AfbRequest, _args: &AfbData, userdata: &mut UnsubscribeData) {
    match SessionUserData::drop(request) {
        Err(mut error) => return request.reply(afb_add_trace!(error), 405),
        Ok(()) => {}
    };

    match userdata.ctx.event.unsubscribe(request) {
        Err(mut error) => request.reply(afb_add_trace!(error), 405),
        Ok(_event) => request.reply(AFB_NO_DATA, 0),
    }
}

struct PushData {
    ctx: Arc<UserCtxData>,
}
AfbVerbRegister!(PushCtrl, push_callback, PushData);
fn push_callback(request: &AfbRequest, args: &AfbData, userdata: &mut PushData) {
    let session = match SessionUserData::get(request) {
        Err(mut error) => return request.reply(afb_add_trace!(error), 405),
        Ok(value) => value,
    };

    session.count += 1;

    let jquery = match args.get::<JsoncObj>(0) {
        Ok(argument) => argument,
        Err(error) => {
            afb_log_msg!(Error, request, "hoop invalid json argument {}", error);
            JsoncObj::from("no-data")
        }
    };

    // increment event counter and push event to listener(s)
    let mut response = AfbParams::new();
    response.push(userdata.ctx.incr_counter()).unwrap();
    response.push(jquery).unwrap();
    let listeners = userdata.ctx.event.push(response);
    request.reply(listeners, 0);
}

struct EvtUserData {
    ctx: Arc<UserCtxData>,
}
AfbEventRegister!(EventGetCtrl, event_get_callback, EvtUserData);
fn event_get_callback(event: &AfbEventMsg, args: &AfbData, userdata: &mut EvtUserData) {
    // check request introspection
    let evt_uid = event.get_uid();
    let evt_name = event.get_name();
    let api_uid = event.get_api().get_uid();

    afb_log_msg!(
        Notice,
        event,
        "--callback evt={} name={} counter={} api={}",
        evt_uid,
        evt_name,
        userdata.ctx.incr_counter(),
        api_uid
    );

    match args.get::<JsoncObj>(0) {
        Ok(argument) => {
            afb_log_msg!(Info, event, "Got valid jsonc object argument={}", argument);
            argument
        }
        Err(error) => {
            afb_log_msg!(Error, event, "hoop invalid json argument {}", error);
            JsoncObj::from("invalid json input argument")
        }
    };
}

// prefix group of event verbs and attach a default privilege
pub fn register(apiv4: AfbApiV4) -> Result<&'static AfbGroup, AfbError> {
    // build verb name from Rust module name
    let mod_name = module_path!().split(':').last().unwrap();
    afb_log_msg!(Notice, apiv4, "Registering group={}", mod_name);

    // create event and build share Arc context data
    let event = AfbEvent::new("demo-event").finalize()?;
    let ctxdata = Arc::new(UserCtxData {
        counter: Cell::new(0),
        event: event,
    });

    let simple_event_handler = AfbEvtHandler::new("handler-1")
        .set_info("My first event handler")
        .set_pattern("helloworld-event/timerCount")
        .set_callback(Box::new(EventGetCtrl {
            ctx: Arc::clone(&ctxdata),
        }))
        .finalize()?;

    let unsubscribe = AfbVerb::new("unsubscribe")
        .set_callback(Box::new(UnsubscribeCtrl {
            ctx: Arc::clone(&ctxdata),
        }))
        .set_info("unsubscribe to event")
        .set_usage("no input")
        .finalize()?;

    let subscribe = AfbVerb::new("subscribe")
        .set_callback(Box::new(SubscribeCtrl {
            ctx: Arc::clone(&ctxdata),
        }))
        .set_info("unsubscribe to event")
        .set_usage("no input")
        .finalize()?;

    let push = AfbVerb::new("push")
        .set_callback(Box::new(PushCtrl {
            ctx: Arc::clone(&ctxdata),
        }))
        .set_info("push query as event output")
        .set_usage("any json data")
        .set_sample("{'skipail':'IoT.bzh'}")?
        .set_permission(AfbPermission::new("acl:evt:push"))
        .finalize()?;

    let group = AfbGroup::new(mod_name)
        .set_info("event demo group")
        .set_prefix(mod_name)
        .set_permission(AfbPermission::new("acl:evt"))
        .set_verbosity(3)
        .add_verb(subscribe)
        .add_verb(unsubscribe)
        .add_verb(push)
        .add_evt_handler(simple_event_handler)
        .add_event(event)
        .finalize()?;
    Ok(group)
}
