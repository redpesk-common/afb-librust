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

#[allow(clippy::arc_with_non_send_sync)]
#[allow(clippy::upper_case_acronyms)]
enum Action {
    SUBSCRIBE,
    UNSUBSCRIBE,
    READ,
    RESET,
}

/// Holds the simulated sensor state.
/// `counter` is a `Cell<u32>` so it can be mutated from both the timer callback and verb callbacks
/// without requiring &mut (it’s interior mutability).
struct UserData {
    event: &'static AfbEvent,
    counter: Cell<u32>,
}

impl UserData {
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

/// Per-verb/timer context. It owns an `Arc<UserData>` so the `UserData` stays alive
/// as long as either the timer or any verb handler still references it.
struct UserContext {
    _debug: &'static str,
    ctx: Arc<UserData>,
}

/// Handle the "sensor" verb. It reads the `UserContext` from `AfbCtxData`,
/// then performs the requested action (subscribe, unsubscribe, read, reset).
fn sensor_cb(request: &AfbRequest, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let ctx = ctx.get_ref::<UserContext>()?.ctx.clone();

    let action = match args.get::<JsoncObj>(0) {
        Err(error) => {
            return Err(afb_add_trace!(error));
        },
        Ok(jquery) => match jquery.get::<String>("action") {
            Err(error) => {
                return Err(afb_add_trace!(error));
            },
            Ok(action) => match action.to_uppercase().as_str() {
                "SUBSCRIBE" => Action::SUBSCRIBE,
                "UNSUBSCRIBE" => Action::UNSUBSCRIBE,
                "READ" => Action::READ,
                "RESET" => Action::RESET,
                _ => {
                    return afb_error!("invalid-action", "expect: SUBSCRIBE|UNSUBSCRIBE|READ|RESET")
                },
            },
        },
    };

    match action {
        Action::SUBSCRIBE => {
            match ctx.event.subscribe(request) {
                Err(error) => request.reply(afb_add_trace!(error), -1),
                Ok(_handle) => request.reply("sensor subscribed", 0),
            };
        },
        Action::UNSUBSCRIBE => {
            match ctx.event.unsubscribe(request) {
                Err(error) => request.reply(afb_add_trace!(error), -1),
                Ok(_handle) => request.reply("sensor unsubscribed", 0),
            };
        },
        Action::READ => {
            request.reply(format!("sensor counter={}", ctx.get_counter()), 0);
        },
        Action::RESET => {
            request.reply(format!("sensor reset={}", ctx.rst_counter()), 0);
        },
    };
    Ok(())
}

/// Timer callback called periodically by AFB.
///
/// IMPORTANT:
/// Do NOT call `ctx.free::<UserContext>()` on every tick for a periodic timer.
/// That would drop the context while the timer is still running, and the next tick
/// would dereference freed memory (use-after-free), causing a crash.
///
/// If you use a finite countdown (via `.set_decount(N)`), you may free the context
/// on the last tick only (when `decount == 1`). For infinite periodic timers,
/// never free in the callback—free when you stop the timer.
fn timer_callback(_timer: &AfbTimer, _decount: u32, ctx: &AfbCtxData) -> Result<(), AfbError> {
    // Borrow the typed context; this returns a shared reference to the stored UserContext.
    let context = ctx.get_ref::<UserContext>()?;

    // Update the counter and publish the new value.
    let count = context.ctx.incr_counter();
    let _listener = context.ctx.event.push(count);

    // WARNING (lifetime & leaks):
    // This timer is periodic and keeps using the UserContext on every tick.
    // Calling `ctx.free::<UserContext>()` here would drop the context while the
    // timer is still alive → the next tick would dereference freed memory
    // (use-after-free / crash). That’s why the free() call is disabled.
    //
    // As a consequence, if the timer runs indefinitely and we never stop it,
    // the UserContext stays allocated for the lifetime of the process. In a
    // long-running service this is acceptable only if you later free it when
    // stopping the timer or tearing down the API.
    //
    // Proper ways to avoid leaks:
    //   1) Finite countdown: if you set `.set_decount(N)`, free only on the last
    //      tick (when `decount == 1`).
    //   2) Explicit stop: call `timer.stop()` somewhere (e.g. in a shutdown hook)
    //      *then* `ctx.free::<UserContext>()`.
    //   3) API shutdown: free the context in your API/group `.on_exit` handler.
    //
    // NOTE: Leaving it allocated here is intentional to avoid UAF, but it will
    //       look like a leak to leak detectors if the timer is never stopped.
    //       Use valgrind/ASan/LSan in a test build and stop the timer before exit
    //       to verify clean teardown.

    // ctx.free::<UserContext>();

    // Example if you use a finite countdown:
    // if _decount == 1 {
    //     // This is the last tick; it is now safe to free the context.
    //     ctx.free::<UserContext>();
    // }

    Ok(())
}

#[allow(clippy::arc_with_non_send_sync)]
pub fn register(apiv4: AfbApiV4) -> Result<&'static AfbGroup, AfbError> {
    // Build verb name from Rust module name
    let mod_name = module_path!().split(':').next_back().unwrap();
    afb_log_msg!(Notice, apiv4, "Registering verb={}", mod_name);

    let event = AfbEvent::new("pub-sub-event");

    // Shared sensor state (counter + event) wrapped in Arc and reused by both timer and verb.
    let ctxdata = Arc::new(UserData { counter: Cell::new(0), event });

    AfbTimer::new("sensor_simulator")
        .set_period(1000)
        .set_callback(timer_callback)
        .set_context(UserContext { _debug: "simulator", ctx: Arc::clone(&ctxdata) })
        // Optional: for a finite number of ticks, uncomment and then free the context
        // on the last tick inside `timer_callback`.
        // .set_decount(10)
        .start()?;

    let verb = AfbVerb::new("pub/sub")
        .set_name("pub-sub")
        .set_callback(sensor_cb)
        .set_context(UserContext { _debug: "pub/sub", ctx: Arc::clone(&ctxdata) })
        .set_actions("['reset','read','subscribe','unsubscribe']")
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
