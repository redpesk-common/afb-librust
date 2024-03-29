/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk samples code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 */

// this example simulate a sensor with a timer. The timer increment a counter at each tic and send an event.
// client may subscribe/unsubscribe to sensor event, read/reset the counter.
// counter is protected with a Cell in order to make it accessible from both the verb callback and the timer.

use afbv4::prelude::*;
use std::cell::Cell;
use std::sync::Arc;
enum Action {
    SUBSCRIBE,
    UNSUBSCRIBE,
    READ,
    RESET,
}

/// Contain data handle to simulate a sensor. Note that count is protected with a Cell in order to be
/// Seen as mutable from both timer and verb callback context.
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

    fn rst_counter(&self) -> u32 {
        self.counter.set(0);
        self.counter.get()
    }
}

/// verb data hold timer data context reference protected with a reference count.
struct UserVcbData {
    ctx: Arc<UserCtxData>,
}

AfbVerbRegister!(PubSubCtrl, sensor_cb, UserVcbData);
fn sensor_cb(
    request: &AfbRequest,
    args: &AfbData,
    userdata: &mut UserVcbData,
) -> Result<(), AfbError> {
    let ctx = userdata.ctx.clone();

    let action = match args.get::<JsoncObj>(0) {
        Err(error) => {
            return Err(afb_add_trace!(error));
        }
        Ok(jquery) => match jquery.get::<String>("action") {
            Err(error) => {
                return Err(afb_add_trace!(error));
            }
            Ok(action) => match action.to_uppercase().as_str() {
                "SUBSCRIBE" => Action::SUBSCRIBE,
                "UNSUBSCRIBE" => Action::UNSUBSCRIBE,
                "READ" => Action::READ,
                "RESET" => Action::RESET,
                _ => {
                    let error= AfbError::new(
                        "invalid-action",
                        "expect: SUBSCRIBE|UNSUBSCRIBE|READ|RESET"
                    );
                    return Err(afb_add_trace!(error))
                }
            },
        },
    };

    match action {
        Action::SUBSCRIBE => {
            match ctx.event.subscribe(request) {
                Err(error) => request.reply(afb_add_trace!(error), -1),
                Ok(_handle) => request.reply("sensor subscribed", 0),
            };
        }
        Action::UNSUBSCRIBE => {
            match ctx.event.unsubscribe(request) {
                Err(error) => request.reply(afb_add_trace!(error), -1),
                Ok(_handle) => request.reply("sensor unsubscribed", 0),
            };
        }
        Action::READ => {
            request.reply(format!("sensor counter={}", ctx.get_counter()), 0);
        }
        Action::RESET => {
            request.reply(format!("sensor reset={}", ctx.rst_counter()), 0);
        }
    };
    Ok(())
}

struct UserTimerData {
    ctx: Arc<UserCtxData>,
}

AfbTimerRegister!(TimerCtrl, timer_callback, UserTimerData);
fn timer_callback(_timer: &AfbTimer, _decount: u32, userdata: &mut UserTimerData) -> Result<(), AfbError>{
    let ctx = userdata.ctx.clone();

    let count = ctx.incr_counter();
    let _listener = ctx.event.push(count);
    Ok(())
}

pub fn register(apiv4: AfbApiV4) -> Result<&'static AfbGroup, AfbError> {
    // build verb name from Rust module name
    let mod_name = module_path!().split(':').last().unwrap();
    afb_log_msg!(Notice, apiv4, "Registering verb={}", mod_name);

    let event = AfbEvent::new("pub-sub-event");
    let ctxdata = Arc::new(UserCtxData {
        counter: Cell::new(0),
        event: event,
    });

    // create an infinite timer that increment a counter and push an event
    let timerdata = UserTimerData {
        ctx: Arc::clone(&ctxdata),
    };

    AfbTimer::new("sensor_simulator")
        .set_period(1000)
        .set_callback(Box::new(timerdata))
        //.set_decount(9999)
        .start()?;

    let vcbdata = UserVcbData {
        ctx: Arc::clone(&ctxdata),
    };
    let verb = AfbVerb::new("pub/sub");
    verb.set_name("pub-sub")
        .set_callback(Box::new(vcbdata))
        .set_action("['reset','read','subscribe','unsubscribe']")
        .expect("valid json array")
        .set_info("simulate publish/subscribe sensor model")
        .set_usage("no input")
        .finalize()?;

    let group = AfbGroup::new(mod_name)
        .set_info("Publish/Subscribe demo group")
        .set_prefix(mod_name)
        .set_permission(AfbPermission::new("acl:pub-sub"))
        .add_verb(verb)
        .add_event(event)
        .finalize()?;

    Ok(group)
}
